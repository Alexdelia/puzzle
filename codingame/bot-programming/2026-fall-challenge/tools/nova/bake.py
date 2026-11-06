import argparse
import os
import pathlib
import shutil
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]


def main():
	parser = argparse.ArgumentParser(
		description="bundle nova for CodinGame with explicit weights instead of weight/nova"
	)
	parser.add_argument("weights", help="exported policy weights (.bin)")
	parser.add_argument(
		"out", help="bundled Rust file; its name must be a valid crate name"
	)
	parser.add_argument(
		"--search-ms",
		type=float,
		default=None,
		help="default one-turn search budget per turn",
	)
	parser.add_argument(
		"--opponent",
		default=None,
		help="exported weights (.bin) predicting the opponent in the search",
	)
	parser.add_argument(
		"--value",
		default=None,
		help="exported weights (.bin) whose value head scores the search candidates",
	)
	parser.add_argument(
		"--set",
		action="append",
		default=[],
		help="NAME=VALUE overriding the f32 constant DEFAULT_NAME",
	)
	args = parser.parse_args()
	settings = [
		f"{name}={float(value)}" for name, value in (s.split("=", 1) for s in args.set)
	]
	if args.search_ms is not None:
		settings.append(f"SEARCH_MS=Some({args.search_ms})")
	with tempfile.TemporaryDirectory() as weight_dir:
		roles = {"policy": args.weights, "opponent": args.opponent, "value": args.value}
		for role, path in roles.items():
			if path is not None:
				shutil.copyfile(path, f"{weight_dir}/{role}.bin")
		subprocess.run(
			[str(ROOT / "solution" / "bundle.sh"), "nova", args.out, *settings],
			check=True,
			env={**os.environ, "WEIGHT_DIR": weight_dir},
		)


if __name__ == "__main__":
	main()
