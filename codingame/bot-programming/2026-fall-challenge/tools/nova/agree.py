import argparse
import os
import pathlib
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import bc
from bc import held_out
from model import NODES, RECORD

ROOT = pathlib.Path(__file__).resolve().parents[2]
LABELLER = ROOT / "target" / "release" / "nova"


def predicted_records(bot, stream, env, scratch):
	answers = scratch / f"{stream.stem}.pred"
	dump = scratch / f"{stream.stem}.bin"
	with open(stream) as source, open(answers, "w") as sink:
		subprocess.run(
			bot,
			stdin=source,
			stdout=sink,
			stderr=subprocess.DEVNULL,
			env=dict(os.environ, **env),
			shell=True,
		)
	labeller_env = {
		key: value for key, value in os.environ.items() if key != "NOVA_WEIGHTS"
	}
	labeller_env.update(NOVA_DUMP=str(dump), NOVA_ANSWERS=str(answers))
	with open(stream) as source:
		subprocess.run(
			[str(LABELLER)],
			stdin=source,
			stdout=subprocess.DEVNULL,
			stderr=subprocess.DEVNULL,
			env=labeller_env,
		)
	return np.fromfile(dump, dtype=RECORD)


def compare(actual, predicted):
	turns = min(len(actual), len(predicted))
	actual, predicted = actual[:turns], predicted[:turns]
	off = ~np.eye(NODES, dtype=bool)
	moves_a = actual["flows"].astype(np.int64) * off
	moves_p = predicted["flows"].astype(np.int64) * off
	labelled = actual["action"] >= 0
	spawn_a = labelled & (actual["action"] >= 1) & (actual["action"] <= 3)
	spawn_p = labelled & (predicted["action"] >= 1) & (predicted["action"] <= 3)
	same_spawn = spawn_a & spawn_p & (actual["action"] == predicted["action"])
	return np.array(
		[
			np.minimum(moves_a, moves_p).sum(),
			np.maximum(moves_a, moves_p).sum(),
			same_spawn.sum(),
			(spawn_a | spawn_p).sum(),
			(labelled & (actual["action"] == predicted["action"])).sum(),
			labelled.sum(),
		],
		dtype=np.float64,
	)


def main():
	parser = argparse.ArgumentParser(
		description="how often a bot's answers match the top players' recorded answers on held-out positions"
	)
	parser.add_argument("bot", help="shell command of the bot")
	parser.add_argument(
		"data", help="directory of <game>-<nick>.bin dumps of the recorded answers"
	)
	parser.add_argument(
		"streams",
		nargs="+",
		help="directories holding the matching <game>-<nick>.in streams",
	)
	parser.add_argument("--holdout", type=float, default=0.2)
	parser.add_argument("--jobs", type=int, default=8)
	parser.add_argument(
		"--all",
		action="store_true",
		help="score every dump, not only held-out top players",
	)
	parser.add_argument(
		"--top", default=None, help="comma list of players held out, replacing TOP"
	)
	args = parser.parse_args()
	if args.top:
		bc.TOP = tuple(args.top.split(","))
	dumps = [
		p
		for p in sorted(pathlib.Path(args.data).glob("*.bin"))
		if args.all or held_out(p, args.holdout)
	]
	streams = {}
	for directory in args.streams:
		for stream in pathlib.Path(directory).glob("*.in"):
			streams[stream.stem] = stream
	with tempfile.TemporaryDirectory() as scratch_name:
		scratch = pathlib.Path(scratch_name)

		def one(dump):
			actual = np.fromfile(dump, dtype=RECORD)
			predicted = predicted_records(args.bot, streams[dump.stem], {}, scratch)
			return compare(actual, predicted)

		with ThreadPoolExecutor(args.jobs) as pool:
			total = sum(pool.map(one, dumps))
	print(
		f"games {len(dumps)} move jaccard {total[0] / max(total[1], 1):.4f} spawn jaccard {total[2] / max(total[3], 1):.4f} triangle action accuracy {total[4] / max(total[5], 1):.4f}"
	)


if __name__ == "__main__":
	main()
