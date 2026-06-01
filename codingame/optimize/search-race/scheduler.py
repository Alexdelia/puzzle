#!/usr/bin/env python3

import os
import subprocess
import sys
import time
from pathlib import Path

SCRIPT_DIR = __file__.rsplit("/", 1)[0]

VALIDATOR_DIR = Path(SCRIPT_DIR) / "validator"
OUTPUT_DIR = Path(SCRIPT_DIR) / "output"
TIME_FILE_NAME = "time.txt"
TURN_TO_FINISH_FILE_NAME = "turn_to_finish.txt"

ITERATION = 100_000

start_with_validator = sys.argv[1] if len(sys.argv) > 1 else None

type ValidatorList = list[tuple[str, Path, int, float | None]]


def build() -> Path:
	subprocess.run(
		[  # noqa: S607
			"cargo",
			"build",
			"--release",
			"--no-default-features",
		],
		env={**dict(os.environ), "ITERATION": str(ITERATION)},
		cwd=SCRIPT_DIR,
		check=True,
	)

	return Path(SCRIPT_DIR) / "target/release/search-race"


def execute(binary: Path, validator_path: Path) -> bool:
	try:
		subprocess.run(  # noqa: S603
			[str(binary), str(validator_path)],
			cwd=SCRIPT_DIR,
			check=True,
		)
	except KeyboardInterrupt:
		print()
		return False
	else:
		return True


def get_validator_list() -> ValidatorList:
	validator_list = []

	for validator in VALIDATOR_DIR.iterdir():
		if validator.is_file() and validator.suffix == ".txt":
			name = validator.stem

			time_file = OUTPUT_DIR / name / TIME_FILE_NAME
			if time_file.exists():
				with time_file.open() as f:
					time = int(f.read().strip())
			else:
				time = 0

			turn_to_finish_file = OUTPUT_DIR / name / TURN_TO_FINISH_FILE_NAME
			if turn_to_finish_file.exists():
				with turn_to_finish_file.open() as f:
					turn_to_finish = float(f.read().strip())
			else:
				turn_to_finish = None

			validator_list.append((name, validator, time, turn_to_finish))

	return validator_list


def update_time(validator_name: str, time: int) -> None:
	time_file = OUTPUT_DIR / validator_name / TIME_FILE_NAME
	time_file.parent.mkdir(parents=True, exist_ok=True)
	with time_file.open("w") as f:
		f.write(str(time))


def sort(validator_list: ValidatorList) -> ValidatorList:
	return sorted(
		validator_list,
		key=lambda x: x[2] + (1_000_000 if x[3] is not None else 0),
	)


def human_readable_time(seconds: int) -> str:
	_sc = "\033[0;34m"
	_mc = "\033[0;36m"
	_hc = "\033[0;38;2;50;168;125m"
	_uc = "\033[2m"

	if seconds < 60:
		return f"{_sc}{seconds}{_uc}s\033[0m"
	if seconds < 3600:
		return f"{_mc}{seconds // 60}{_uc}m {_sc}{seconds % 60}{_uc}s\033[0m"
	return f"{_hc}{seconds // 3600}{_uc}h {_mc}{(seconds % 3600) // 60}{_uc}m\033[0m"


binary = build()

vl = get_validator_list()
vl = sort(vl)

if start_with_validator:
	for i, (vn, *_) in enumerate(vl):
		if vn == start_with_validator:
			vl.insert(0, vl.pop(i))
			break
	else:
		print(
			f"\033[1;31merror\033[0m: validator '{start_with_validator}' not found\n"
			"available validators:"
			+ "".join(f"\n  - \033[1;32m{vn}\033[0m" for vn, _, _, _ in vl)
		)
		sys.exit(64)  # EX_USAGE


print()
for vn, _, vt, vtf in vl:
	success_mark = "\033[1;32m✓\033[0m" if vtf is not None else "-"

	vtf_str = f"{vtf:.2f}" if vtf is not None else ""

	print(
		f"  {success_mark} \033[32m{vn:<6}"
		f" \033[0;1m{vtf_str:>6}"
		f" \033[0m{human_readable_time(vt)}"
	)
print()


while True:
	[vn, vp, vt, vtf] = vl.pop(0)

	print(f"\033[1;32m{vn}\033[0m")

	start = time.perf_counter()

	success = execute(binary, vp)

	end = time.perf_counter()
	elapsed = int(end - start)
	vt += elapsed

	print(f"{human_readable_time(elapsed)} ({human_readable_time(vt)})\n")

	update_time(vn, vt)

	if not success:
		break

	turn_to_finish_file = OUTPUT_DIR / vn / TURN_TO_FINISH_FILE_NAME
	if turn_to_finish_file.exists():
		with turn_to_finish_file.open() as f:
			vtf = float(f.read().strip())

	vl.append((vn, vp, vt, vtf))
	vl = sort(vl)
