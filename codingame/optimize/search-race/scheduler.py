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

ITERATION = 10_000


def build() -> Path:
	subprocess.run(
		["cargo", "build", "--release", "--no-default-features"],  # noqa: S607
		env={**dict(os.environ), "ITERATION": str(ITERATION)},
		cwd=SCRIPT_DIR,
		check=True,
	)

	return Path(SCRIPT_DIR) / "target/release/search-race"


def exexcute(binary: Path, validator_path: Path) -> None:
	try:
		subprocess.run(  # noqa: S603
			[str(binary), str(validator_path)],
			cwd=SCRIPT_DIR,
			check=True,
		)
	except KeyboardInterrupt:
		sys.exit(0)


def get_validator_list() -> list[tuple[str, Path, int]]:
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

			validator_list.append((name, validator, time))

	return validator_list


def update_time(validator_name: str, time: int) -> None:
	time_file = OUTPUT_DIR / validator_name / TIME_FILE_NAME
	time_file.parent.mkdir(parents=True, exist_ok=True)
	with time_file.open("w") as f:
		f.write(str(time))


def sort_by_time(
	validator_list: list[tuple[str, Path, int]],
) -> list[tuple[str, Path, int]]:
	return sorted(validator_list, key=lambda x: x[2])


def human_readable_time(seconds: int) -> str:
	if seconds < 60:
		return f"{seconds}s"
	if seconds < 3600:
		return f"{seconds // 60}m {seconds % 60}s"
	return f"{seconds // 3600}h {(seconds % 3600) // 60}m {seconds % 60}s"


binary = build()

vl = get_validator_list()
vl = sort_by_time(vl)


print()
for vn, _, vt in vl:
	print(f"- \033[32m{vn}\033[0m: \033[36m{human_readable_time(vt)}\033[0m")
print()


while True:
	[vn, vp, vt] = vl.pop(0)

	print(f"\033[1;32m{vn}\033[0m")

	start = time.perf_counter()

	exexcute(binary, vp)

	end = time.perf_counter()
	elapsed = int(end - start)
	vt += elapsed

	print(
		f"\033[36m{human_readable_time(elapsed)}\033[0m"
		f" (\033[36m{human_readable_time(vt)}\033[0m)\n"
	)

	update_time(vn, vt)

	vl.append((vn, vp, vt))
	vl = sort_by_time(vl)
