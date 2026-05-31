#!/usr/bin/env python3

import sys
import os
import subprocess
import time
from pathlib import Path

SCRIPT_DIR = __file__.rsplit("/", 1)[0]

VALIDATOR_DIR = Path(SCRIPT_DIR) / "validator"
OUTPUT_DIR = Path(SCRIPT_DIR) / "output"
TIME_FILE_NAME = "time.txt"
SUCCESS_FILE_NAME = "success.flag"

ITERATION = 100_000

start_with_validator = sys.argv[1] if len(sys.argv) > 1 else None

type ValidatorList = list[tuple[str, Path, int, bool]]


def build() -> Path:
	subprocess.run(
		[  # noqa: S607
			"cargo",
			"build",
			"--bin",
			"naive-genetic-algorithm",
			"--release",
			"--no-default-features",
		],
		env={**dict(os.environ), "ITERATION": str(ITERATION)},
		cwd=SCRIPT_DIR,
		check=True,
	)

	return Path(SCRIPT_DIR) / "target/release/naive-genetic-algorithm"


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

			success_file = OUTPUT_DIR / name / SUCCESS_FILE_NAME
			success = success_file.exists()

			validator_list.append((name, validator, time, success))

	return validator_list


def mark_success(validator_name: str) -> None:
	success_file = OUTPUT_DIR / validator_name / SUCCESS_FILE_NAME
	success_file.parent.mkdir(parents=True, exist_ok=True)
	success_file.touch()


def update_time(validator_name: str, time: int) -> None:
	time_file = OUTPUT_DIR / validator_name / TIME_FILE_NAME
	time_file.parent.mkdir(parents=True, exist_ok=True)
	with time_file.open("w") as f:
		f.write(str(time))


def sort(validator_list: ValidatorList) -> ValidatorList:
	return sorted(validator_list, key=lambda x: x[2] + (1_000_000 if x[3] else 0))


def human_readable_time(seconds: int) -> str:
	if seconds < 60:
		return f"{seconds}s"
	if seconds < 3600:
		return f"{seconds // 60}m {seconds % 60}s"
	return f"{seconds // 3600}h {(seconds % 3600) // 60}m {seconds % 60}s"


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
for vn, _, vt, vs in vl:
	success_mark = "\033[1;32m✓\033[0m" if vs else "-"
	print(
		f"  {success_mark} \033[32m{vn}\033[0m:"
		f" \033[36m{human_readable_time(vt)}\033[0m"
	)
print()


while True:
	[vn, vp, vt, vs] = vl.pop(0)

	print(f"\033[1;32m{vn}\033[0m")

	start = time.perf_counter()

	success = execute(binary, vp)
	if success:
		vs = True
		mark_success(vn)

	end = time.perf_counter()
	elapsed = int(end - start)
	vt += elapsed

	print(
		f"\033[36m{human_readable_time(elapsed)}\033[0m"
		f" (\033[36m{human_readable_time(vt)}\033[0m)\n"
	)

	update_time(vn, vt)

	if not success:
		break

	vl.append((vn, vp, vt, vs))
	vl = sort(vl)
