use contest::broker::{
	Allocation, Grant, JobClass, JobRequest, Load, Machine, RunningJob, allocate,
	parse_mem_available_mb,
};
use contest::paths::JobId;

fn machine() -> Machine {
	Machine {
		physical: vec![vec![0, 4], vec![1, 5], vec![2, 6], vec![3, 7]],
		gpus: 1,
		ram_budget_mb: 13000,
		agent_cores: 1,
	}
}

const TIMED_THREADS: [usize; 6] = [0, 1, 2, 4, 5, 6];
const AGENT_THREADS: [usize; 2] = [3, 7];

fn job(id: &str, class: JobClass) -> JobRequest {
	JobRequest {
		id: JobId(id.to_string()),
		class,
		gpu: false,
		ram_mb: 1000,
		priority: 0,
	}
}

fn timed(id: &str) -> JobRequest {
	job(
		id,
		JobClass::Timed {
			min_cores: 1,
			max_cores: 4,
		},
	)
}

fn batch(id: &str) -> JobRequest {
	job(id, JobClass::Batch)
}

fn fixed_timed(id: &str, cores: usize, priority: i32) -> JobRequest {
	JobRequest {
		priority,
		..job(
			id,
			JobClass::Timed {
				min_cores: cores,
				max_cores: cores,
			},
		)
	}
}

fn holding(request: JobRequest, cpus: &[usize]) -> RunningJob {
	RunningJob {
		request,
		cpus: cpus.to_vec(),
		draining_to: None,
	}
}

fn draining(request: JobRequest, cpus: &[usize], to: &[usize]) -> RunningJob {
	RunningJob {
		draining_to: Some(to.to_vec()),
		..holding(request, cpus)
	}
}

fn idle() -> Load {
	Load::default()
}

fn swapping() -> Load {
	Load {
		swapping: true,
		..Load::default()
	}
}

fn grant<'a>(allocation: &'a Allocation, id: &str) -> &'a Grant {
	allocation
		.grants
		.iter()
		.find(|grant| grant.id.0 == id)
		.unwrap_or_else(|| panic!("no grant for {id}"))
}

fn ids(names: &[&str]) -> Vec<JobId> {
	names.iter().map(|name| JobId(name.to_string())).collect()
}

fn assert_no_empty_cpus(allocation: &Allocation) {
	assert!(allocation.grants.iter().all(|grant| !grant.cpus.is_empty()));
}

fn assert_disjoint(a: &[usize], b: &[usize]) {
	assert!(a.iter().all(|cpu| !b.contains(cpu)), "{a:?} overlaps {b:?}");
}

#[test]
fn lone_batch_gets_all_threads() {
	let allocation = allocate(&machine(), &[], &[batch("j1")], &idle());
	let granted = grant(&allocation, "j1");
	assert_eq!(granted.cpus, (0..8).collect::<Vec<_>>());
	assert!(!granted.exclusive);
	assert_eq!(allocation.agent_cpus, (0..8).collect::<Vec<_>>());
}

#[test]
fn lone_timed_gets_every_core_but_the_agent_core() {
	let allocation = allocate(&machine(), &[], &[timed("j1")], &idle());
	let granted = grant(&allocation, "j1");
	assert_eq!(granted.cpus, TIMED_THREADS);
	assert!(granted.exclusive);
	assert_eq!(allocation.agent_cpus, AGENT_THREADS);
}

#[test]
fn agent_cpus_exclude_every_timed_core() {
	let running = [holding(timed("j1"), &[1, 5]), holding(batch("j2"), &[0, 4])];
	let allocation = allocate(&machine(), &running, &[timed("j3")], &idle());
	let mut timed_cpus: Vec<usize> = ["j1", "j3"]
		.iter()
		.flat_map(|id| grant(&allocation, id).cpus.clone())
		.collect();
	timed_cpus.sort_unstable();
	assert_eq!(timed_cpus, TIMED_THREADS);
	assert_eq!(allocation.agent_cpus, AGENT_THREADS);
	assert_eq!(grant(&allocation, "j2").cpus, allocation.agent_cpus);
}

#[test]
fn no_jobs_leave_every_cpu_to_agents() {
	let allocation = allocate(&machine(), &[], &[], &idle());
	assert!(allocation.grants.is_empty());
	assert_eq!(allocation.agent_cpus, (0..8).collect::<Vec<_>>());
}

#[test]
fn timed_and_batch_split() {
	let allocation = allocate(&machine(), &[], &[timed("j1"), batch("j2")], &idle());
	assert_eq!(grant(&allocation, "j1").cpus, TIMED_THREADS);
	assert_eq!(grant(&allocation, "j2").cpus, AGENT_THREADS);
}

#[test]
fn two_queued_timed_share_round_robin() {
	let allocation = allocate(&machine(), &[], &[timed("j1"), timed("j2")], &idle());
	assert_eq!(grant(&allocation, "j1").cpus.len(), 4);
	assert_eq!(grant(&allocation, "j2").cpus.len(), 2);
	assert_eq!(allocation.admitted.len(), 2);
}

#[test]
fn running_timed_job_keeps_its_cores_and_only_grows() {
	let running = [holding(timed("j1"), &[2, 6])];
	let allocation = allocate(&machine(), &running, &[], &idle());
	let cpus = &grant(&allocation, "j1").cpus;
	assert!(cpus.contains(&2) && cpus.contains(&6), "{cpus:?}");
	assert_eq!(cpus, &TIMED_THREADS);
	assert!(allocation.shrinks.is_empty());
}

#[test]
fn arriving_timed_job_waits_for_a_drain() {
	let running = [holding(timed("j1"), &TIMED_THREADS)];
	let allocation = allocate(&machine(), &running, &[timed("j2")], &idle());
	assert!(allocation.admitted.is_empty());
	assert_eq!(grant(&allocation, "j1").cpus, TIMED_THREADS);
	assert_eq!(
		allocation.shrinks,
		vec![(ids(&["j1"])[0].clone(), vec![0, 1, 4, 5])]
	);
	assert_eq!(allocation.agent_cpus, AGENT_THREADS);
}

#[test]
fn draining_job_is_never_shrunk_twice() {
	let running = [draining(timed("j1"), &TIMED_THREADS, &[0, 1, 4, 5])];
	let allocation = allocate(&machine(), &running, &[timed("j2")], &idle());
	assert!(allocation.admitted.is_empty());
	assert!(allocation.shrinks.is_empty());
	assert_eq!(grant(&allocation, "j1").cpus, TIMED_THREADS);
}

#[test]
fn drained_cores_go_to_the_waiting_job_not_back_to_the_donor() {
	let running = [holding(timed("j1"), &[0, 1, 4, 5])];
	let allocation = allocate(&machine(), &running, &[timed("j2")], &idle());
	assert_eq!(allocation.admitted, ids(&["j2"]));
	assert_eq!(grant(&allocation, "j1").cpus, [0, 1, 4, 5]);
	assert_eq!(grant(&allocation, "j2").cpus, [2, 6]);
	assert!(allocation.shrinks.is_empty());
}

#[test]
fn free_cores_stay_reserved_while_a_drain_runs() {
	let running = [
		draining(timed("j1"), &[0, 1, 4, 5], &[0, 4]),
		holding(fixed_timed("j2", 1, 0), &[2, 6]),
	];
	let queued = [fixed_timed("j3", 2, 5), timed("j4")];
	let allocation = allocate(&machine(), &running, &queued, &idle());
	assert!(allocation.admitted.is_empty(), "{allocation:?}");
	assert_eq!(grant(&allocation, "j1").cpus, [0, 1, 4, 5]);
	assert_eq!(grant(&allocation, "j2").cpus, [2, 6]);
}

#[test]
fn shrink_takes_only_the_cores_the_waiting_job_needs() {
	let running = [
		holding(timed("j1"), &[0, 1, 4, 5]),
		holding(timed("j2"), &[2, 6]),
	];
	let allocation = allocate(&machine(), &running, &[timed("j3")], &idle());
	assert_eq!(
		allocation.shrinks,
		vec![(ids(&["j1"])[0].clone(), vec![0, 4])]
	);
	assert!(allocation.admitted.is_empty());
}

#[test]
fn batch_never_takes_cores_from_running_timed_jobs() {
	let running = [holding(fixed_timed("j1", 3, 0), &TIMED_THREADS)];
	let allocation = allocate(&machine(), &running, &[batch("j2")], &idle());
	assert_eq!(allocation.admitted, ids(&["j2"]));
	assert_eq!(grant(&allocation, "j1").cpus, TIMED_THREADS);
	assert_eq!(grant(&allocation, "j2").cpus, AGENT_THREADS);
	assert!(allocation.shrinks.is_empty());
	assert_no_empty_cpus(&allocation);
}

#[test]
fn queued_timed_below_min_cores_stays_queued() {
	let running = [holding(fixed_timed("j1", 3, 0), &TIMED_THREADS)];
	let allocation = allocate(&machine(), &running, &[timed("j2")], &idle());
	assert!(allocation.admitted.is_empty());
	assert!(allocation.shrinks.is_empty());
	assert_eq!(allocation.grants.len(), 1);
}

#[test]
fn gpu_exclusive() {
	let gpu_job = |id: &str| JobRequest {
		gpu: true,
		..batch(id)
	};
	let allocation = allocate(&machine(), &[], &[gpu_job("j1"), gpu_job("j2")], &idle());
	assert_eq!(allocation.admitted, ids(&["j1"]));
}

#[test]
fn ram_admission() {
	let sized = |id: &str, ram_mb| JobRequest {
		ram_mb,
		..batch(id)
	};
	let allocation = allocate(
		&machine(),
		&[],
		&[sized("j1", 8000), sized("j2", 6000)],
		&idle(),
	);
	assert_eq!(allocation.admitted, ids(&["j1"]));
}

#[test]
fn ram_admission_reserves_room_for_running_agents() {
	let big = JobRequest {
		ram_mb: 12000,
		..batch("j1")
	};
	let two_agents = Load {
		agent_ram_mb: 1400,
		..Load::default()
	};
	let refused = allocate(&machine(), &[], std::slice::from_ref(&big), &two_agents);
	assert!(refused.admitted.is_empty());
	let one_agent = Load {
		agent_ram_mb: 700,
		..Load::default()
	};
	let admitted = allocate(&machine(), &[], &[big], &one_agent);
	assert_eq!(admitted.admitted, ids(&["j1"]));
}

#[test]
fn ram_admission_never_exceeds_available_memory() {
	let short = Load {
		mem_available_mb: Some(1500),
		..Load::default()
	};
	let allocation = allocate(&machine(), &[], &[batch("j1"), batch("j2")], &short);
	assert_eq!(allocation.admitted, ids(&["j1"]));
	let running = [holding(batch("j0"), &AGENT_THREADS)];
	let allocation = allocate(&machine(), &running, &[batch("j1")], &short);
	assert_eq!(allocation.admitted, ids(&["j1"]));
}

#[test]
fn mem_available_parses_kib_into_mib() {
	let meminfo =
		"MemTotal:       16311000 kB\nMemFree:         3608000 kB\nMemAvailable:    8766680 kB\n";
	assert_eq!(parse_mem_available_mb(meminfo), Some(8561));
	assert_eq!(parse_mem_available_mb("MemTotal: 1 kB\n"), None);
}

#[test]
fn queued_ordered_by_priority_then_id() {
	let prioritised = |id: &str, priority| JobRequest {
		priority,
		ram_mb: 8000,
		..batch(id)
	};
	let queued = [
		prioritised("j1", 0),
		prioritised("j2", 5),
		prioritised("j3", 5),
	];
	let allocation = allocate(&machine(), &[], &queued, &idle());
	assert_eq!(allocation.admitted, ids(&["j2"]));
}

#[test]
fn no_timed_while_swapping() {
	let running = [holding(timed("j1"), &[0, 4])];
	let allocation = allocate(&machine(), &running, &[timed("j2")], &swapping());
	assert!(allocation.admitted.is_empty());
	assert!(grant(&allocation, "j1").contended);
}

#[test]
fn not_contended_when_not_swapping() {
	let running = [holding(timed("j1"), &[0, 4])];
	let allocation = allocate(&machine(), &running, &[], &idle());
	assert!(!grant(&allocation, "j1").contended);
}

#[test]
fn batch_still_admitted_while_swapping() {
	let allocation = allocate(&machine(), &[], &[batch("j1")], &swapping());
	assert_eq!(allocation.admitted.len(), 1);
}

#[test]
fn cpu_list_parses_pairs_and_ranges() {
	use contest::broker::parse_cpu_list;
	assert_eq!(parse_cpu_list("0,4\n"), vec![0, 4]);
	assert_eq!(parse_cpu_list("0-1"), vec![0, 1]);
	assert_eq!(parse_cpu_list("0-1,4-5"), vec![0, 1, 4, 5]);
}

#[test]
fn detect_finds_cores() {
	let machine = Machine::detect(13000, 1);
	assert!(!machine.physical.is_empty());
	assert_eq!(machine.timed_cores(), machine.physical.len() - 1);
}

#[test]
fn at_least_one_core_stays_with_agents() {
	let no_reserve = Machine {
		agent_cores: 0,
		..machine()
	};
	assert_eq!(no_reserve.timed_cores(), 3);
	let allocation = allocate(&no_reserve, &[], &[timed("j1")], &idle());
	assert_eq!(allocation.agent_cpus, AGENT_THREADS);
}

#[test]
fn high_priority_timed_and_low_priority_batch_both_run() {
	let queued = [batch("j2"), fixed_timed("j1", 3, 10)];
	let allocation = allocate(&machine(), &[], &queued, &idle());
	assert_eq!(allocation.admitted, ids(&["j1", "j2"]));
	assert_eq!(grant(&allocation, "j1").cpus, TIMED_THREADS);
	assert_eq!(grant(&allocation, "j2").cpus, AGENT_THREADS);
	assert_no_empty_cpus(&allocation);
}

#[test]
fn single_core_timed_jobs_holding_every_timed_core_leave_batch_the_agent_core() {
	let running: Vec<RunningJob> = (0..3)
		.map(|core| holding(fixed_timed(&format!("j{core}"), 1, 0), &[core, core + 4]))
		.collect();
	let allocation = allocate(&machine(), &running, &[batch("j9")], &idle());
	assert_eq!(allocation.admitted, ids(&["j9"]));
	assert_eq!(grant(&allocation, "j9").cpus, AGENT_THREADS);
	assert_no_empty_cpus(&allocation);
}

#[test]
fn rejected_gpu_timed_does_not_block_later_gpu_batch() {
	let running = [holding(fixed_timed("j1", 3, 0), &TIMED_THREADS)];
	let rejected = JobRequest {
		gpu: true,
		..fixed_timed("j2", 2, 9)
	};
	let later = JobRequest {
		gpu: true,
		..batch("j3")
	};
	let allocation = allocate(&machine(), &running, &[rejected, later], &idle());
	assert_eq!(allocation.admitted, ids(&["j3"]));
	assert!(grant(&allocation, "j3").gpu);
}

#[test]
fn rejected_timed_ram_not_consumed_for_later_job() {
	let running = [holding(fixed_timed("j1", 3, 0), &TIMED_THREADS)];
	let rejected = JobRequest {
		ram_mb: 10000,
		..fixed_timed("j2", 2, 9)
	};
	let later = JobRequest {
		ram_mb: 10000,
		..batch("j3")
	};
	let allocation = allocate(&machine(), &running, &[rejected, later], &idle());
	assert_eq!(allocation.admitted, ids(&["j3"]));
}

#[test]
fn invalid_timed_never_admitted() {
	let zero_min = job(
		"j1",
		JobClass::Timed {
			min_cores: 0,
			max_cores: 2,
		},
	);
	let inverted = job(
		"j2",
		JobClass::Timed {
			min_cores: 3,
			max_cores: 2,
		},
	);
	let allocation = allocate(&machine(), &[], &[zero_min, inverted], &idle());
	assert!(allocation.admitted.is_empty());
	assert!(allocation.grants.is_empty());
}

#[test]
fn mixed_scenario_never_emits_empty_or_shared_timed_cpus() {
	let running = [holding(timed("j1"), &[0, 4]), holding(batch("j2"), &[3, 7])];
	let queued = [fixed_timed("j3", 2, 5), timed("j4"), batch("j5")];
	let allocation = allocate(&machine(), &running, &queued, &idle());
	assert_no_empty_cpus(&allocation);
	let timed_grants: Vec<&Grant> = allocation
		.grants
		.iter()
		.filter(|grant| grant.exclusive)
		.collect();
	for (index, first) in timed_grants.iter().enumerate() {
		assert_disjoint(&first.cpus, &allocation.agent_cpus);
		for second in &timed_grants[index + 1..] {
			assert_disjoint(&first.cpus, &second.cpus);
		}
	}
}

#[test]
fn malformed_cpu_list_parts_are_dropped() {
	use contest::broker::parse_cpu_list;
	assert_eq!(parse_cpu_list("0,x,2-y,3"), vec![0, 3]);
}
