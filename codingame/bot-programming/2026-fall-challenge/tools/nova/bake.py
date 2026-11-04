import argparse
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from model import pack

ROOT = pathlib.Path(__file__).resolve().parents[2]


def main():
	parser = argparse.ArgumentParser(
		description="bundle nova for CodinGame with a weights file packed into WEIGHTS"
	)
	parser.add_argument("weights", help="exported weights (.bin)")
	parser.add_argument(
		"out", help="bundled Rust file; its name must be a valid crate name"
	)
	parser.add_argument(
		"--search-ms",
		type=float,
		default=None,
		help="default one-turn search budget per turn",
	)
	args = parser.parse_args()
	subprocess.run(
		[str(ROOT / "solution" / "bundle.sh"), "nova", args.out],
		check=True,
		stdout=subprocess.DEVNULL,
	)
	source = pathlib.Path(args.out)
	text = pack(pathlib.Path(args.weights).read_bytes())
	body, replaced = re.subn(
		r'pub const WEIGHTS: &str = "[^"]*";',
		f'pub const WEIGHTS: &str = "{text}";',
		source.read_text(encoding="utf-8"),
		count=1,
	)
	assert replaced == 1
	if args.search_ms is not None:
		body, replaced = re.subn(
			r"pub const DEFAULT_SEARCH_MS: Option<f64> = None;",
			f"pub const DEFAULT_SEARCH_MS: Option<f64> = Some({args.search_ms});",
			body,
			count=1,
		)
		assert replaced == 1
	source.write_text(body, encoding="utf-8")
	units = len(body.encode("utf-16-le")) // 2
	print(f"{source}: {units} UTF-16 units, weights {len(text)} characters")
	assert units <= 100000, "over the CodinGame limit"


if __name__ == "__main__":
	main()
