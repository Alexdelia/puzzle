import argparse
import os
import re

import numpy as np

from train import ATTACK, MAX_DEGREE, NODE_FEATURES, RECORD

NODES = 50


def keep_mask(turns, every, stride):
	return (turns <= every) | (turns % stride == 0)


def main():
	parser = argparse.ArgumentParser(
		description="pack extracted comet records into compact memory-mapped arrays"
	)
	parser.add_argument(
		"--records", help="directory of extracted .rec files, labelled by --labels"
	)
	parser.add_argument("--labels", help="tab-separated stream name and outcome")
	parser.add_argument(
		"--arena",
		nargs="*",
		default=[],
		help="directories of arena games recorded by comet, with results.txt",
	)
	parser.add_argument("--out", required=True)
	parser.add_argument(
		"--every", type=int, default=20, help="keep every turn up to this one"
	)
	parser.add_argument(
		"--stride", type=int, default=5, help="then keep one turn in this many"
	)
	args = parser.parse_args()
	outcome = {}
	paths = {}
	if args.labels:
		for line in open(args.labels):
			name, value = line.rstrip("\n").split("\t")
			path = os.path.join(args.records, name + ".rec")
			if os.path.exists(path):
				outcome[name] = float(value)
				paths[name] = path
	for d, directory in enumerate(args.arena):
		for line in open(os.path.join(directory, "results.txt")):
			m = re.match(r"game (\d+) seed (\d+) swapped \w+ winner (\S+)", line)
			if not m:
				continue
			index, seed, winner = int(m.group(1)), int(m.group(2)), m.group(3)
			for side in (0, 1):
				path = os.path.join(directory, f"{index}-p{side + 1}.rec")
				if not os.path.exists(path):
					continue
				value = (
					0.0
					if winner == "None"
					else (1.0 if winner == f"Some({side})" else -1.0)
				)
				name = f"{1_000_000_000 + d * 100_000 + index}-{side}"
				outcome[name] = value
				paths[name] = path
	names = sorted(outcome)
	total = 0
	for name in names:
		turns = np.fromfile(paths[name], dtype=RECORD)["turn"]
		total += int(keep_mask(turns, args.every, args.stride).sum())
	print(f"streams {len(names)} records {total}", flush=True)
	os.makedirs(args.out, exist_ok=True)
	shapes = {
		"nodes": ((total, NODES, NODE_FEATURES), np.float16),
		"globals": ((total, 8), np.float32),
		"neighbors": ((total, NODES, MAX_DEGREE), np.int8),
		"units": ((total, NODES), np.int16),
		"moves": ((total, NODES, MAX_DEGREE + 1), np.uint8),
		"corners": ((total, 32, 3), np.int8),
		"labels": ((total, 32), np.int16),
		"masks": ((total, 32), np.int64),
		"value": ((total,), np.float32),
		"game": ((total,), np.int64),
		"turn": ((total,), np.int32),
		"count": ((total,), np.int32),
		"source": ((total,), np.int32),
	}
	arrays = {
		k: np.lib.format.open_memmap(
			os.path.join(args.out, k + ".npy"), mode="w+", dtype=d, shape=s
		)
		for k, (s, d) in shapes.items()
	}
	at = 0
	for i, name in enumerate(names):
		raw = np.fromfile(paths[name], dtype=RECORD)
		raw = raw[keep_mask(raw["turn"], args.every, args.stride)]
		if len(raw) == 0:
			continue
		assert raw["count"].max() <= NODES
		end = at + len(raw)
		tri = raw["triangles"]
		label = tri["label"].astype(np.int64)
		cls = np.where(label == ATTACK, ATTACK + tri["target"].astype(np.int64), label)
		cls = np.where(label < 0, -1, cls)
		arrays["nodes"][at:end] = raw["nodes"][:, :NODES].astype(np.float16)
		arrays["globals"][at:end] = raw["globals"]
		arrays["neighbors"][at:end] = raw["neighbors"][:, :NODES]
		arrays["units"][at:end] = raw["units"][:, :NODES]
		arrays["moves"][at:end] = np.clip(raw["moves"][:, :NODES], 0, 255).astype(
			np.uint8
		)
		arrays["corners"][at:end] = (
			tri["corners"].astype(np.int16).clip(-1, 127).astype(np.int8)
		)
		arrays["labels"][at:end] = cls.astype(np.int16)
		arrays["masks"][at:end] = tri["mask"].view(np.int64)
		arrays["value"][at:end] = outcome[name]
		arrays["game"][at:end] = int(name.split("-", 1)[0])
		arrays["turn"][at:end] = raw["turn"]
		arrays["count"][at:end] = raw["count"]
		arrays["source"][at:end] = raw["pad"]
		at = end
		if i % 5000 == 0:
			print(f"{i} streams, {at} records", flush=True)
	for a in arrays.values():
		a.flush()
	print(f"wrote {at} records to {args.out}")


if __name__ == "__main__":
	main()
