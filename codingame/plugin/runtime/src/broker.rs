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
			Some((first, last)) => match (first.parse::<usize>(), last.parse::<usize>()) {
				(Ok(first), Ok(last)) => (first..=last).collect::<Vec<_>>(),
				_ => Vec::new(),
			},
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
	let mut admission = Admission::new(machine);
	for job in running.iter().filter(|job| is_valid(job)) {
		admission.keep_running(job);
	}
	for job in queue_order(queued) {
		if is_valid(job) && admission.admits_queued(job, swapping) {
			admission.add(job, false);
		}
	}
	admission.into_allocation(machine, swapping)
}

fn is_batch(job: &JobRequest) -> bool {
	matches!(job.class, JobClass::Batch)
}

fn is_valid(job: &JobRequest) -> bool {
	match job.class {
		JobClass::Batch => true,
		JobClass::Timed {
			min_cores,
			max_cores,
		} => min_cores > 0 && max_cores >= min_cores,
	}
}

fn min_cores_of(job: &JobRequest) -> usize {
	match job.class {
		JobClass::Timed { min_cores, .. } => min_cores,
		JobClass::Batch => 0,
	}
}

fn queue_order(queued: &[JobRequest]) -> Vec<&JobRequest> {
	let mut ordered: Vec<&JobRequest> = queued.iter().collect();
	ordered.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.id.cmp(&b.id)));
	ordered
}

struct Admission<'a> {
	cores: usize,
	ram_budget_mb: u64,
	gpu_slots: usize,
	jobs: Vec<(&'a JobRequest, bool)>,
	ram_used_mb: u64,
	gpus_used: usize,
	reserved_cores: usize,
	batch_admitted: bool,
}

impl<'a> Admission<'a> {
	fn new(machine: &Machine) -> Self {
		Admission {
			cores: machine.physical.len(),
			ram_budget_mb: machine.ram_budget_mb,
			gpu_slots: machine.gpus,
			jobs: Vec::new(),
			ram_used_mb: 0,
			gpus_used: 0,
			reserved_cores: 0,
			batch_admitted: false,
		}
	}

	fn keep_running(&mut self, job: &'a JobRequest) {
		self.add(job, true);
	}

	fn add(&mut self, job: &'a JobRequest, was_running: bool) {
		self.ram_used_mb += job.ram_mb;
		self.gpus_used += usize::from(job.gpu);
		self.reserved_cores += min_cores_of(job);
		self.batch_admitted |= is_batch(job);
		self.jobs.push((job, was_running));
	}

	fn admits_queued(&self, job: &JobRequest, swapping: bool) -> bool {
		self.fits_ram(job)
			&& self.fits_gpu(job)
			&& !blocked_by_swap(job, swapping)
			&& self.fits_cores(job)
	}

	fn fits_ram(&self, job: &JobRequest) -> bool {
		self.ram_used_mb + job.ram_mb <= self.ram_budget_mb
	}

	fn fits_gpu(&self, job: &JobRequest) -> bool {
		!job.gpu || self.gpus_used < self.gpu_slots
	}

	fn fits_cores(&self, job: &JobRequest) -> bool {
		if is_batch(job) {
			return self.reserved_cores < self.cores;
		}
		let kept_for_batch = if self.batch_admitted {
			MIN_CORES_FOR_BATCH
		} else {
			0
		};
		self.reserved_cores + min_cores_of(job) + kept_for_batch <= self.cores
	}

	fn timed_core_budget(&self) -> usize {
		let kept_for_batch = if self.batch_admitted {
			MIN_CORES_FOR_BATCH
		} else {
			0
		};
		self.cores.saturating_sub(kept_for_batch)
	}

	fn into_allocation(self, machine: &Machine, swapping: bool) -> Allocation {
		let mut holdings = self.give_minimum_cores();
		let held: usize = holdings.iter().map(|holding| holding.cores.len()).sum();
		let extra = self.timed_core_budget().saturating_sub(held);
		share_extra_round_robin(&mut holdings, held, extra);
		let held: usize = holdings.iter().map(|holding| holding.cores.len()).sum();
		let batch_threads = threads_of(machine, &(held..self.cores).collect::<Vec<_>>());

		let mut grants = Vec::new();
		let mut admitted = Vec::new();
		for (job, was_running) in &self.jobs {
			let cpus = if is_batch(job) {
				batch_threads.clone()
			} else {
				holdings
					.iter()
					.find(|holding| holding.id == job.id)
					.map(|holding| threads_of(machine, &holding.cores))
					.unwrap_or_default()
			};
			if cpus.is_empty() {
				continue;
			}
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

	fn give_minimum_cores(&self) -> Vec<Holding> {
		let mut next_core = 0;
		let mut holdings = Vec::new();
		for (job, _) in &self.jobs {
			let JobClass::Timed {
				min_cores,
				max_cores,
			} = job.class
			else {
				continue;
			};
			holdings.push(Holding {
				id: job.id.clone(),
				max_cores,
				cores: (next_core..next_core + min_cores).collect(),
			});
			next_core += min_cores;
		}
		holdings
	}
}

fn blocked_by_swap(job: &JobRequest, swapping: bool) -> bool {
	swapping && !is_batch(job)
}

struct Holding {
	id: JobId,
	max_cores: usize,
	cores: Vec<usize>,
}

fn share_extra_round_robin(holdings: &mut [Holding], first_free_core: usize, extra: usize) {
	let mut next_core = first_free_core;
	let end = first_free_core + extra;
	loop {
		let mut progressed = false;
		for holding in holdings.iter_mut() {
			if holding.cores.len() < holding.max_cores && next_core < end {
				holding.cores.push(next_core);
				next_core += 1;
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
		.flat_map(|&core| machine.physical.get(core).into_iter().flatten().copied())
		.collect();
	threads.sort_unstable();
	threads
}
