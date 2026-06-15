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
COMPLETE_FLAG_NAME = "complete.flag"

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


def is_complete(validator_name: str) -> bool:
	return (OUTPUT_DIR / validator_name / COMPLETE_FLAG_NAME).exists()


def resolve_validator(token: str) -> str | None:
	name_list = sorted(path.stem for path in VALIDATOR_DIR.glob("*.txt"))
	if token in name_list:
		return token
	if token.isdigit():
		prefix = f"{int(token):02d}_"
		return next((name for name in name_list if name.startswith(prefix)), None)
	return None


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


def human_readable_time(seconds: int, dim: bool = False) -> str:
	level = "2;" if dim else ""
	if seconds < 60:
		return f"\033[0;{level}96m{seconds}\033[2ms\033[0m"
	if seconds < 3600:
		return (
			f"\033[0;{level}36m{seconds // 60}\033[2mm "
			f"\033[0;{level}96m{seconds % 60}\033[2ms\033[0m"
		)
	return (
		f"\033[0;{level}34m{seconds // 3600}\033[2mh"
		f" \033[0;{level}36m{(seconds % 3600) // 60}\033[2mm\033[0m"
	)


def format_entry(validator_name: str, elapsed: int, score: int, complete: bool) -> str:
	prefix = "✓" if complete else " "
	dim = "\033[2m" if complete else ""
	weight = "2" if complete else "1"
	return (
		f"{dim}  {prefix} \033[32m{validator_name:<28}"
		f"\033[0;{weight}m{score:>6}\033[0m  "
		f"{human_readable_time(elapsed, dim=complete)}"
	)


def total_score(validator_list: ValidatorList) -> int:
	return sum(entry[3] for entry in validator_list)


binary = build()

full_list = sort(get_validator_list())
active_list = [entry for entry in full_list if not is_complete(entry[0])]
complete_list = [entry for entry in full_list if is_complete(entry[0])]

if start_with_validator:
	target = resolve_validator(start_with_validator) or start_with_validator
	for i, (vn, *_) in enumerate(active_list):
		if vn == target:
			active_list.insert(0, active_list.pop(i))
			break

print()
for vn, _, vt, vs in active_list:
	print(format_entry(vn, vt, vs, complete=False))
for vn, _, vt, vs in complete_list:
	print(format_entry(vn, vt, vs, complete=True))
print(f"\n  total \033[1;32m{total_score(full_list)}\033[0m\n")

if not active_list:
	print("  \033[2mnothing to search\033[0m\n")
	sys.exit(0)

visit_count: dict[str, int] = {}

while True:
	[vn, vp, vt, vs] = active_list.pop(0)
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

	active_list.append((vn, vp, vt, vs))
	active_list = sort(active_list)
