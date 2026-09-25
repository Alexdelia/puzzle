#!/usr/bin/env python3

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RESULT = ROOT / ".2048_results.out"
ANSWER = ROOT / "src" / "answer.rs"
LIMIT = 100_000


def declaration(line: str) -> str:
	return line.split(" = ", 1)[0]


def pack() -> dict[str, str]:
	out = subprocess.run(  # noqa: S603
		["cargo", "run", "--release", "--quiet", "--bin", "pack", "--", str(RESULT)],  # noqa: S607
		cwd=ROOT,
		check=True,
		capture_output=True,
		text=True,
	)
	sys.stderr.write(out.stderr)
	return {declaration(line): line for line in out.stdout.splitlines()}


def merge(template: str, constants: dict[str, str]) -> str:
	lines = template.splitlines(keepends=True)
	missing = set(constants)
	for i, line in enumerate(lines):
		key = declaration(line)
		if key in constants:
			lines[i] = constants[key] + "\n"
			missing.discard(key)
	if missing:
		absent = sorted(missing)
		sys.exit(f"{ANSWER} lack {absent}")
	return "".join(lines)


def utf16_units(text: str) -> int:
	return len(text.encode("utf-16-le")) // 2


if __name__ == "__main__":
	merged = merge(ANSWER.read_text(), pack())
	units = utf16_units(merged)
	print(f"{units} / {LIMIT} UTF-16", file=sys.stderr)
	if units > LIMIT:
		sys.exit("over codingame limit")
	sys.stdout.write(merged)
