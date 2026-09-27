import os
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

VERIFY = "target/release/verify"


def run(solver, level, timeout):
	start = time.perf_counter()
	try:
		out = subprocess.run(
			[solver],
			stdin=level.open(),
			capture_output=True,
			timeout=timeout,
			text=True,
		)
	except subprocess.TimeoutExpired:
		return level, None, timeout
	elapsed = time.perf_counter() - start
	with tempfile.NamedTemporaryFile("w", suffix=".out", delete=False) as solution:
		solution.write(out.stdout)
	ok = (
		subprocess.run(
			[VERIFY, str(level), solution.name], capture_output=True
		).returncode
		== 0
	)
	os.unlink(solution.name)
	return level, ok, elapsed


def main():
	solver, timeout, *patterns = sys.argv[1:]
	levels = sorted(p for pattern in patterns for p in Path("level").glob(pattern))
	jobs = int(os.environ.get("JOBS", "1"))
	with ThreadPoolExecutor(jobs) as pool:
		results = list(
			pool.map(lambda level: run(solver, level, float(timeout)), levels)
		)
	for level, ok, elapsed in results:
		status = {None: "TIMEOUT", True: "ok", False: "WRONG"}[ok]
		print(f"{level.stem:12} {status:8} {elapsed:8.3f}s")
	times = sorted(elapsed for _, _, elapsed in results)
	solved = sum(ok is True for _, ok, _ in results)
	print(
		f"solved {solved}/{len(levels)}  median {times[len(times) // 2]:.3f}s  max {times[-1]:.3f}s"
	)


main()
