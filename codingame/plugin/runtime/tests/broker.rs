use contest::broker::{Allocation, Grant, JobClass, JobRequest, Machine, allocate};
use contest::paths::JobId;

fn machine() -> Machine {
	Machine {
		physical: vec![vec![0, 4], vec![1, 5], vec![2, 6], vec![3, 7]],
		gpus: 1,
		ram_budget_mb: 13000,
	}
}

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

fn grant<'a>(allocation: &'a Allocation, id: &str) -> &'a Grant {
	allocation
		.grants
		.iter()
		.find(|grant| grant.id.0 == id)
		.unwrap_or_else(|| panic!("no grant for {id}"))
}

#[test]
fn lone_batch_gets_all_threads() {
	let allocation = allocate(&machine(), &[], &[batch("j1")], false);
	let granted = grant(&allocation, "j1");
	assert_eq!(granted.cpus, (0..8).collect::<Vec<_>>());
	assert!(!granted.exclusive);
}

#[test]
fn lone_timed_gets_all_cores() {
	let allocation = allocate(&machine(), &[], &[timed("j1")], false);
	let granted = grant(&allocation, "j1");
	assert_eq!(granted.cpus, (0..8).collect::<Vec<_>>());
	assert!(granted.exclusive);
}

#[test]
fn timed_and_batch_split() {
	let allocation = allocate(&machine(), &[], &[timed("j1"), batch("j2")], false);
	assert_eq!(grant(&allocation, "j1").cpus, vec![0, 1, 2, 4, 5, 6]);
	assert_eq!(grant(&allocation, "j2").cpus, vec![3, 7]);
}

#[test]
fn two_timed_share_round_robin() {
	let allocation = allocate(&machine(), &[], &[timed("j1"), timed("j2")], false);
	assert_eq!(grant(&allocation, "j1").cpus.len(), 4);
	assert_eq!(grant(&allocation, "j2").cpus.len(), 4);
	assert_eq!(allocation.admitted.len(), 2);
}

#[test]
fn queued_timed_below_min_cores_stays_queued() {
	let greedy = job(
		"j1",
		JobClass::Timed {
			min_cores: 4,
			max_cores: 4,
		},
	);
	let allocation = allocate(&machine(), &[greedy], &[timed("j2")], false);
	assert!(allocation.admitted.is_empty());
	assert_eq!(allocation.grants.len(), 1);
}

#[test]
fn gpu_exclusive() {
	let gpu_job = |id: &str| JobRequest {
		gpu: true,
		..batch(id)
	};
	let allocation = allocate(&machine(), &[], &[gpu_job("j1"), gpu_job("j2")], false);
	assert_eq!(allocation.admitted, vec![JobId("j1".to_string())]);
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
		false,
	);
	assert_eq!(allocation.admitted, vec![JobId("j1".to_string())]);
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
	let allocation = allocate(&machine(), &[], &queued, false);
	assert_eq!(allocation.admitted, vec![JobId("j2".to_string())]);
}

#[test]
fn no_timed_while_swapping() {
	let allocation = allocate(&machine(), &[timed("j1")], &[timed("j2")], true);
	assert!(allocation.admitted.is_empty());
	assert!(grant(&allocation, "j1").contended);
}

#[test]
fn not_contended_when_not_swapping() {
	let allocation = allocate(&machine(), &[timed("j1")], &[], false);
	assert!(!grant(&allocation, "j1").contended);
}

#[test]
fn batch_still_admitted_while_swapping() {
	let allocation = allocate(&machine(), &[], &[batch("j1")], true);
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
	let machine = Machine::detect(13000);
	assert!(!machine.physical.is_empty());
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

fn assert_no_empty_cpus(allocation: &Allocation) {
	assert!(allocation.grants.iter().all(|grant| !grant.cpus.is_empty()));
}

#[test]
fn high_priority_timed_beats_low_priority_batch() {
	let queued = [batch("j2"), fixed_timed("j1", 4, 10)];
	let allocation = allocate(&machine(), &[], &queued, false);
	assert_eq!(allocation.admitted, vec![JobId("j1".to_string())]);
	assert_eq!(grant(&allocation, "j1").cpus.len(), 8);
	assert_no_empty_cpus(&allocation);
}

#[test]
fn running_timed_keeps_min_cores_against_queued_batch() {
	let running = [fixed_timed("j1", 4, 0)];
	let allocation = allocate(&machine(), &running, &[batch("j2")], false);
	assert!(allocation.admitted.is_empty());
	assert_eq!(grant(&allocation, "j1").cpus.len(), 8);
	assert_no_empty_cpus(&allocation);
}

#[test]
fn four_running_single_core_timed_leave_no_room_for_batch() {
	let running: Vec<JobRequest> = (1..=4)
		.map(|n| fixed_timed(&format!("j{n}"), 1, 0))
		.collect();
	let allocation = allocate(&machine(), &running, &[batch("j9")], false);
	assert!(allocation.admitted.is_empty());
	assert_eq!(allocation.grants.len(), 4);
	assert_no_empty_cpus(&allocation);
}

#[test]
fn rejected_gpu_timed_does_not_block_later_gpu_batch() {
	let running = [fixed_timed("j1", 3, 0)];
	let rejected = JobRequest {
		gpu: true,
		..fixed_timed("j2", 2, 9)
	};
	let later = JobRequest {
		gpu: true,
		..batch("j3")
	};
	let allocation = allocate(&machine(), &running, &[rejected, later], false);
	assert_eq!(allocation.admitted, vec![JobId("j3".to_string())]);
	assert!(grant(&allocation, "j3").gpu);
}

#[test]
fn rejected_timed_ram_not_consumed_for_later_job() {
	let running = [fixed_timed("j1", 3, 0)];
	let rejected = JobRequest {
		ram_mb: 10000,
		..fixed_timed("j2", 2, 9)
	};
	let later = JobRequest {
		ram_mb: 10000,
		..batch("j3")
	};
	let allocation = allocate(&machine(), &running, &[rejected, later], false);
	assert_eq!(allocation.admitted, vec![JobId("j3".to_string())]);
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
	let allocation = allocate(&machine(), &[], &[zero_min, inverted], false);
	assert!(allocation.admitted.is_empty());
	assert!(allocation.grants.is_empty());
}

#[test]
fn mixed_scenario_never_emits_empty_cpus() {
	let running = [timed("j1"), batch("j2")];
	let queued = [fixed_timed("j3", 3, 5), timed("j4"), batch("j5")];
	let allocation = allocate(&machine(), &running, &queued, false);
	assert_no_empty_cpus(&allocation);
}

#[test]
fn malformed_cpu_list_parts_are_dropped() {
	use contest::broker::parse_cpu_list;
	assert_eq!(parse_cpu_list("0,x,2-y,3"), vec![0, 3]);
}
