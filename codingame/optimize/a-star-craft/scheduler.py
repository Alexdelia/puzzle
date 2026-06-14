#!/usr/bin/env python3

import os
import subprocess
import sys
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent

VALIDATOR_DIR = SCRIPT_DIR / "validator"
OUTPUT_DIR = SCRIPT_DIR / "output"
TIME_FILE_NAME = "time.txt"
SCORE_FILE_NAME = "score.txt"

DURATION = os.environ.get("DURATION", "30")
STRATEGY_CYCLE = os.environ.get("STRATEGY_CYCLE", "sa,ils").split(",")

start_with_validator = sys.argv[1] if len(sys.argv) > 1 else None

type ValidatorList = list[tuple[str, Path, int, int]]


def build() -> Path:
	subprocess.run(
		["cargo", "build", "--release"],  # noqa: S607
		cwd=SCRIPT_DIR,
		check=True,
	)
	return SCRIPT_DIR / "target/release/a-star-craft"


def read_int(path: Path) -> int:
	if path.exists():
		return int(path.read_text().strip())
	return 0


def execute(binary: Path, validator_path: Path, strategy: str) -> bool:
	try:
		subprocess.run(  # noqa: S603
			[str(binary), str(validator_path)],
			cwd=SCRIPT_DIR,
			env={**os.environ, "DURATION": DURATION, "STRATEGY": strategy},
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
			elapsed = read_int(OUTPUT_DIR / name / TIME_FILE_NAME)
			score = read_int(OUTPUT_DIR / name / SCORE_FILE_NAME)
			validator_list.append((name, validator, elapsed, score))
	return validator_list


def update_time(validator_name: str, elapsed: int) -> None:
	time_file = OUTPUT_DIR / validator_name / TIME_FILE_NAME
	time_file.parent.mkdir(parents=True, exist_ok=True)
	time_file.write_text(str(elapsed))


def sort(validator_list: ValidatorList) -> ValidatorList:
	return sorted(validator_list, key=lambda entry: entry[2])


def human_readable_time(seconds: int) -> str:
	if seconds < 60:
		return f"\033[0;96m{seconds}\033[2ms\033[0m"
	if seconds < 3600:
		return (
			f"\033[0;36m{seconds // 60}\033[2mm \033[0;96m{seconds % 60}\033[2ms\033[0m"
		)
	return (
		f"\033[0;34m{seconds // 3600}\033[2mh"
		f" \033[0;36m{(seconds % 3600) // 60}\033[2mm\033[0m"
	)


def total_score(validator_list: ValidatorList) -> int:
	return sum(entry[3] for entry in validator_list)


binary = build()

vl = sort(get_validator_list())

if start_with_validator:
	for i, (vn, *_) in enumerate(vl):
		if vn == start_with_validator:
			vl.insert(0, vl.pop(i))
			break

print()
for vn, _, vt, vs in vl:
	mark = "\033[1;32m✓\033[0m" if vs > 0 else "-"
	print(
		f"  {mark} \033[32m{vn:<28}\033[0;1m{vs:>6}\033[0m  {human_readable_time(vt)}"
	)
print(f"\n  total \033[1;32m{total_score(vl)}\033[0m\n")

visit_count: dict[str, int] = {}

while True:
	[vn, vp, vt, vs] = vl.pop(0)
	strategy = STRATEGY_CYCLE[visit_count.get(vn, 0) % len(STRATEGY_CYCLE)]
	visit_count[vn] = visit_count.get(vn, 0) + 1

	print(f"\033[1;32m{vn}\033[0m \033[2m{strategy}\033[0m")
	start = time.perf_counter()
	success = execute(binary, vp, strategy)
	elapsed = int(time.perf_counter() - start)
	vt += elapsed
	update_time(vn, vt)

	vs = read_int(OUTPUT_DIR / vn / SCORE_FILE_NAME)
	print(
		f"  \033[0;1m{vs}\033[0m"
		f"  {human_readable_time(elapsed)}"
		f"({human_readable_time(vt)})\n"
	)

	if not success:
		break

	vl.append((vn, vp, vt, vs))
	vl = sort(vl)
