use std::fs;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::paths::JobId;

const MIN_CORES_FOR_BATCH: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machine {
	pub physical: Vec<Vec<usize>>,
	pub gpus: usize,
	pub ram_budget_mb: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobClass {
	Timed { min_cores: usize, max_cores: usize },
	Batch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRequest {
	pub id: JobId,
	pub class: JobClass,
	pub gpu: bool,
	pub ram_mb: u64,
	pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
	pub id: JobId,
	pub cpus: Vec<usize>,
	pub exclusive: bool,
	pub gpu: bool,
	pub contended: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
	pub grants: Vec<Grant>,
	pub admitted: Vec<JobId>,
}

impl Machine {
	pub fn detect(ram_budget_mb: u64) -> Machine {
		Machine {
			physical: detect_physical_cores(),
			gpus: detect_gpus(),
			ram_budget_mb,
		}
	}
}

fn detect_physical_cores() -> Vec<Vec<usize>> {
	let mut groups: Vec<Vec<usize>> = Vec::new();
	let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") else {
		return groups;
	};
	for entry in entries.flatten() {
		let path = entry.path().join("topology/thread_siblings_list");
		let Ok(text) = fs::read_to_string(path) else {
			continue;
		};
		let mut group = parse_cpu_list(&text);
		group.sort_unstable();
		if !group.is_empty() && !groups.contains(&group) {
			groups.push(group);
		}
	}
	groups.sort();
	groups
}

pub fn parse_cpu_list(text: &str) -> Vec<usize> {
	text.trim()
		.split(',')
		.filter(|part| !part.is_empty())
		.flat_map(|part| match part.split_once('-') {
			Some((first, last)) => {
				let first: usize = first.parse().unwrap_or(0);
				let last: usize = last.parse().unwrap_or(0);
				(first..=last).collect::<Vec<_>>()
			}
			None => part.parse().into_iter().collect(),
		})
		.collect()
}

fn detect_gpus() -> usize {
	Command::new("nvidia-smi")
		.arg("-L")
		.output()
		.ok()
		.filter(|output| output.status.success())
		.map(|output| String::from_utf8_lossy(&output.stdout).lines().count())
		.unwrap_or(0)
}

pub fn allocate(
	machine: &Machine,
	running: &[JobRequest],
	queued: &[JobRequest],
	swapping: bool,
) -> Allocation {
	let queue = queue_order(queued);
	let admission = admit_by_resources(machine, running, &queue, swapping);
	let any_batch = admission.iter().any(|(job, _)| is_batch(job));
	let cores = split_cores(machine, &admission, any_batch);

	let mut admitted = Vec::new();
	let mut grants = Vec::new();
	for (job, was_running) in &admission {
		let Some(cpus) = cores.cpus_of(job) else {
			continue;
		};
		if !was_running {
			admitted.push(job.id.clone());
		}
		grants.push(Grant {
			id: job.id.clone(),
			cpus,
			exclusive: !is_batch(job),
			gpu: job.gpu,
			contended: *was_running && swapping && !is_batch(job),
		});
	}
	Allocation { grants, admitted }
}

fn is_batch(job: &JobRequest) -> bool {
	matches!(job.class, JobClass::Batch)
}

fn queue_order(queued: &[JobRequest]) -> Vec<&JobRequest> {
	let mut ordered: Vec<&JobRequest> = queued.iter().collect();
	ordered.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.id.cmp(&b.id)));
	ordered
}

fn admit_by_resources<'a>(
	machine: &Machine,
	running: &'a [JobRequest],
	queue: &[&'a JobRequest],
	swapping: bool,
) -> Vec<(&'a JobRequest, bool)> {
	let mut admission: Vec<(&JobRequest, bool)> = running.iter().map(|job| (job, true)).collect();
	for &job in queue {
		let fits = fits_ram(machine, &admission, job)
			&& fits_gpu(machine, &admission, job)
			&& !blocked_by_swap(job, swapping);
		if fits {
			admission.push((job, false));
		}
	}
	admission
}

fn fits_ram(machine: &Machine, admission: &[(&JobRequest, bool)], job: &JobRequest) -> bool {
	let used: u64 = admission.iter().map(|(admitted, _)| admitted.ram_mb).sum();
	used + job.ram_mb <= machine.ram_budget_mb
}

fn fits_gpu(machine: &Machine, admission: &[(&JobRequest, bool)], job: &JobRequest) -> bool {
	let used = admission
		.iter()
		.filter(|(admitted, _)| admitted.gpu)
		.count();
	!job.gpu || used < machine.gpus
}

fn blocked_by_swap(job: &JobRequest, swapping: bool) -> bool {
	swapping && !is_batch(job)
}

struct CoreSplit {
	timed: Vec<(JobId, Vec<usize>)>,
	batch: Vec<usize>,
}

impl CoreSplit {
	fn cpus_of(&self, job: &JobRequest) -> Option<Vec<usize>> {
		if is_batch(job) {
			return Some(self.batch.clone());
		}
		self.timed
			.iter()
			.find(|(id, _)| *id == job.id)
			.map(|(_, cpus)| cpus.clone())
	}
}

fn split_cores(machine: &Machine, admission: &[(&JobRequest, bool)], any_batch: bool) -> CoreSplit {
	let timed_budget = timed_core_budget(machine, any_batch);
	let mut free: Vec<usize> = (0..timed_budget).collect();
	let mut holdings = give_minimum_cores(admission, &mut free);
	share_remaining_round_robin(&mut holdings, &mut free);

	let timed = holdings
		.iter()
		.map(|holding| (holding.id.clone(), threads_of(machine, &holding.cores)))
		.collect();
	let held: usize = holdings.iter().map(|holding| holding.cores.len()).sum();
	let batch_cores: Vec<usize> = (held..machine.physical.len()).collect();
	CoreSplit {
		timed,
		batch: threads_of(machine, &batch_cores),
	}
}

fn timed_core_budget(machine: &Machine, any_batch: bool) -> usize {
	let reserved = if any_batch { MIN_CORES_FOR_BATCH } else { 0 };
	machine.physical.len().saturating_sub(reserved)
}

struct Holding {
	id: JobId,
	max_cores: usize,
	cores: Vec<usize>,
}

fn give_minimum_cores(admission: &[(&JobRequest, bool)], free: &mut Vec<usize>) -> Vec<Holding> {
	let mut holdings = Vec::new();
	for (job, was_running) in admission {
		let JobClass::Timed {
			min_cores,
			max_cores,
		} = job.class
		else {
			continue;
		};
		if free.len() < min_cores && !was_running {
			continue;
		}
		let taken = min_cores.min(free.len());
		holdings.push(Holding {
			id: job.id.clone(),
			max_cores,
			cores: free.drain(..taken).collect(),
		});
	}
	holdings
}

fn share_remaining_round_robin(holdings: &mut [Holding], free: &mut Vec<usize>) {
	loop {
		let mut progressed = false;
		for holding in holdings.iter_mut() {
			if holding.cores.len() < holding.max_cores && !free.is_empty() {
				holding.cores.push(free.remove(0));
				progressed = true;
			}
		}
		if !progressed {
			return;
		}
	}
}

fn threads_of(machine: &Machine, cores: &[usize]) -> Vec<usize> {
	let mut threads: Vec<usize> = cores
		.iter()
		.flat_map(|&core| machine.physical[core].iter().copied())
		.collect();
	threads.sort_unstable();
	threads
}
