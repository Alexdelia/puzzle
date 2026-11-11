use std::fs;
use std::process::Command;

use anyhow::ensure;
use serde::{Deserialize, Serialize};

use crate::paths::JobId;

const MEMINFO_PATH: &str = "/proc/meminfo";
const MEM_AVAILABLE: &str = "MemAvailable:";
const KIB_PER_MIB: u64 = 1024;
const MIN_AGENT_CORES: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machine {
	pub physical: Vec<Vec<usize>>,
	pub gpus: usize,
	pub ram_budget_mb: u64,
	pub agent_cores: usize,
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
pub struct RunningJob {
	pub request: JobRequest,
	pub cpus: Vec<usize>,
	pub draining_to: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Load {
	pub swapping: bool,
	pub agent_ram_mb: u64,
	pub mem_available_mb: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
	pub grants: Vec<Grant>,
	pub admitted: Vec<JobId>,
	pub shrinks: Vec<(JobId, Vec<usize>)>,
	pub agent_cpus: Vec<usize>,
}

impl Machine {
	pub fn detect(ram_budget_mb: u64, agent_cores: usize) -> Machine {
		Machine {
			physical: detect_physical_cores(),
			gpus: detect_gpus(),
			ram_budget_mb,
			agent_cores,
		}
	}

	pub fn timed_cores(&self) -> usize {
		self.physical
			.len()
			.saturating_sub(self.agent_cores.max(MIN_AGENT_CORES))
	}

	pub fn all_cpus(&self) -> Vec<usize> {
		threads_of(self, &(0..self.physical.len()).collect::<Vec<_>>())
	}

	pub fn check_admittable(&self, request: &JobRequest) -> anyhow::Result<()> {
		if let JobClass::Timed { min_cores, .. } = request.class {
			ensure!(
				min_cores <= self.timed_cores(),
				"min_cores {min_cores} can never be granted: timed jobs hold at most {timed} of the {physical} physical cores ({agents} kept for agents and builds)",
				timed = self.timed_cores(),
				physical = self.physical.len(),
				agents = self.physical.len() - self.timed_cores()
			);
		}
		ensure!(
			request.ram_mb <= self.ram_budget_mb,
			"ram_mb {ram} is above the RAM budget of {budget} MB",
			ram = request.ram_mb,
			budget = self.ram_budget_mb
		);
		ensure!(
			!request.gpu || self.gpus > 0,
			"gpu requested but this machine has no GPU"
		);
		Ok(())
	}
}

pub fn physical_core_count() -> usize {
	detect_physical_cores().len()
}

pub fn parse_mem_available_mb(meminfo: &str) -> Option<u64> {
	meminfo
		.lines()
		.find_map(|line| line.strip_prefix(MEM_AVAILABLE))
		.and_then(|rest| rest.split_whitespace().next()?.parse::<u64>().ok())
		.map(|kib| kib / KIB_PER_MIB)
}

pub fn mem_available_mb() -> Option<u64> {
	parse_mem_available_mb(&fs::read_to_string(MEMINFO_PATH).ok()?)
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
	running: &[RunningJob],
	queued: &[JobRequest],
	load: &Load,
) -> Allocation {
	let mut plan = Plan::new(machine, load);
	for job in running.iter().filter(|job| is_valid(&job.request)) {
		plan.keep_running(job);
	}
	for job in queue_order(queued) {
		if is_valid(job) {
			plan.consider(job);
		}
	}
	plan.into_allocation()
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

fn queue_order(queued: &[JobRequest]) -> Vec<&JobRequest> {
	let mut ordered: Vec<&JobRequest> = queued.iter().collect();
	ordered.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.id.cmp(&b.id)));
	ordered
}

struct TimedHolding {
	id: JobId,
	min_cores: usize,
	max_cores: usize,
	priority: i32,
	cores: Vec<usize>,
	keep: Option<Vec<usize>>,
	shrink_planned: bool,
	was_running: bool,
	gpu: bool,
}

impl TimedHolding {
	fn kept_cores(&self) -> usize {
		self.keep.as_ref().unwrap_or(&self.cores).len()
	}

	fn can_shrink(&self) -> bool {
		self.was_running
			&& (self.keep.is_none() || self.shrink_planned)
			&& self.kept_cores() > self.min_cores
	}

	fn can_grow(&self) -> bool {
		self.keep.is_none() && self.cores.len() < self.max_cores
	}

	fn give_up_one_core(&mut self) {
		let keep = self.keep.get_or_insert_with(|| self.cores.clone());
		keep.sort_unstable();
		keep.pop();
		self.shrink_planned = true;
	}
}

struct BatchHolding {
	id: JobId,
	gpu: bool,
}

enum Holding {
	Timed(TimedHolding),
	Batch(BatchHolding),
}

struct Plan<'a> {
	machine: &'a Machine,
	load: &'a Load,
	holdings: Vec<Holding>,
	admitted: Vec<JobId>,
	ram_used_mb: u64,
	ram_started_mb: u64,
	gpus_used: usize,
	free: Vec<usize>,
	coming: Vec<usize>,
}

fn cores_of(machine: &Machine, cpus: &[usize]) -> Vec<usize> {
	machine
		.physical
		.iter()
		.enumerate()
		.filter(|(_, threads)| threads.iter().any(|thread| cpus.contains(thread)))
		.map(|(core, _)| core)
		.collect()
}

impl<'a> Plan<'a> {
	fn new(machine: &'a Machine, load: &'a Load) -> Self {
		Plan {
			machine,
			load,
			holdings: Vec::new(),
			admitted: Vec::new(),
			ram_used_mb: 0,
			ram_started_mb: 0,
			gpus_used: 0,
			free: (0..machine.timed_cores()).collect(),
			coming: Vec::new(),
		}
	}

	fn keep_running(&mut self, job: &RunningJob) {
		let request = &job.request;
		self.ram_used_mb += request.ram_mb;
		self.gpus_used += usize::from(request.gpu);
		let JobClass::Timed {
			min_cores,
			max_cores,
		} = request.class
		else {
			self.holdings.push(Holding::Batch(BatchHolding {
				id: request.id.clone(),
				gpu: request.gpu,
			}));
			return;
		};
		let cores = cores_of(self.machine, &job.cpus);
		let keep = job
			.draining_to
			.as_ref()
			.map(|cpus| cores_of(self.machine, cpus));
		self.free.retain(|core| !cores.contains(core));
		if let Some(keep) = &keep {
			self.coming
				.extend(cores.iter().filter(|core| !keep.contains(core)));
		}
		self.holdings.push(Holding::Timed(TimedHolding {
			id: request.id.clone(),
			min_cores,
			max_cores,
			priority: request.priority,
			cores,
			keep,
			shrink_planned: false,
			was_running: true,
			gpu: request.gpu,
		}));
	}

	fn fits_ram(&self, job: &JobRequest) -> bool {
		self.ram_used_mb + self.load.agent_ram_mb + job.ram_mb <= self.machine.ram_budget_mb
			&& self
				.load
				.mem_available_mb
				.is_none_or(|available| self.ram_started_mb + job.ram_mb <= available)
	}

	fn fits_gpu(&self, job: &JobRequest) -> bool {
		!job.gpu || self.gpus_used < self.machine.gpus
	}

	fn reserve(&mut self, job: &JobRequest) {
		self.ram_used_mb += job.ram_mb;
		self.ram_started_mb += job.ram_mb;
		self.gpus_used += usize::from(job.gpu);
	}

	fn consider(&mut self, job: &JobRequest) {
		if !self.fits_ram(job) || !self.fits_gpu(job) {
			return;
		}
		let JobClass::Timed {
			min_cores,
			max_cores,
		} = job.class
		else {
			self.reserve(job);
			self.admitted.push(job.id.clone());
			self.holdings.push(Holding::Batch(BatchHolding {
				id: job.id.clone(),
				gpu: job.gpu,
			}));
			return;
		};
		if self.load.swapping {
			return;
		}
		if self.free.len() >= min_cores {
			self.reserve(job);
			let cores: Vec<usize> = self.free.drain(..min_cores).collect();
			self.admitted.push(job.id.clone());
			self.holdings.push(Holding::Timed(TimedHolding {
				id: job.id.clone(),
				min_cores,
				max_cores,
				priority: job.priority,
				cores,
				keep: None,
				shrink_planned: false,
				was_running: false,
				gpu: job.gpu,
			}));
			return;
		}
		let missing = min_cores - self.free.len();
		let from_coming = missing.min(self.coming.len());
		let to_reclaim = missing - from_coming;
		if self.shrinkable_cores() < to_reclaim {
			return;
		}
		self.plan_shrinks(to_reclaim);
		self.reserve(job);
		self.free.clear();
		self.coming.drain(..from_coming);
	}

	fn timed_mut(&mut self) -> impl Iterator<Item = &mut TimedHolding> {
		self.holdings
			.iter_mut()
			.filter_map(|holding| match holding {
				Holding::Timed(timed) => Some(timed),
				Holding::Batch(_) => None,
			})
	}

	fn timed(&self) -> impl Iterator<Item = &TimedHolding> {
		self.holdings.iter().filter_map(|holding| match holding {
			Holding::Timed(timed) => Some(timed),
			Holding::Batch(_) => None,
		})
	}

	fn shrinkable_cores(&self) -> usize {
		self.timed()
			.filter(|holding| holding.can_shrink())
			.map(|holding| holding.kept_cores() - holding.min_cores)
			.sum()
	}

	fn plan_shrinks(&mut self, cores: usize) {
		for _ in 0..cores {
			let donor = self
				.timed_mut()
				.filter(|holding| holding.can_shrink())
				.max_by(|a, b| {
					(a.kept_cores() - a.min_cores)
						.cmp(&(b.kept_cores() - b.min_cores))
						.then_with(|| b.priority.cmp(&a.priority))
						.then_with(|| a.id.cmp(&b.id))
				});
			if let Some(donor) = donor {
				donor.give_up_one_core();
			}
		}
	}

	fn grow_round_robin(&mut self) {
		let mut free = std::mem::take(&mut self.free).into_iter();
		loop {
			let mut grew = false;
			for holding in self.timed_mut().filter(|holding| holding.can_grow()) {
				let Some(core) = free.next() else {
					return;
				};
				holding.cores.push(core);
				grew = true;
			}
			if !grew {
				return;
			}
		}
	}

	fn into_allocation(mut self) -> Allocation {
		self.grow_round_robin();
		let machine = self.machine;
		let held: Vec<usize> = self
			.timed()
			.flat_map(|holding| holding.cores.iter().copied())
			.collect();
		let untimed: Vec<usize> = (0..machine.physical.len())
			.filter(|core| !held.contains(core))
			.collect();
		let agent_cpus = threads_of(machine, &untimed);
		let swapping = self.load.swapping;
		let mut grants = Vec::new();
		let mut shrinks = Vec::new();
		for holding in &self.holdings {
			match holding {
				Holding::Timed(timed) => {
					if timed.shrink_planned
						&& let Some(keep) = &timed.keep
					{
						shrinks.push((timed.id.clone(), threads_of(machine, keep)));
					}
					grants.push(Grant {
						id: timed.id.clone(),
						cpus: threads_of(machine, &timed.cores),
						exclusive: true,
						gpu: timed.gpu,
						contended: timed.was_running && swapping,
					});
				}
				Holding::Batch(batch) => grants.push(Grant {
					id: batch.id.clone(),
					cpus: agent_cpus.clone(),
					exclusive: false,
					gpu: batch.gpu,
					contended: false,
				}),
			}
		}
		grants.retain(|grant| !grant.cpus.is_empty());
		let admitted = self
			.admitted
			.into_iter()
			.filter(|id| grants.iter().any(|grant| &grant.id == id))
			.collect();
		Allocation {
			grants,
			admitted,
			shrinks,
			agent_cpus,
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
