import argparse
import pathlib
import sys

import torch

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from model import NODE_FEATURES, GLOBAL_FEATURES, TRIANGLE_FEATURES, Nova

NOISE = 0.02
NEW_ROUND_BIAS = 1e-3


def widen(
	old, new, old_in_blocks, new_in_blocks, zero_new_outputs, zero_new_inputs=True
):
	with torch.no_grad():
		new.weight.normal_(0.0, NOISE)
		new.bias.zero_()
		out_old = old.weight.shape[0]
		if zero_new_outputs:
			new.weight[out_old:] = 0.0
		for (old_start, old_size), (new_start, new_size) in zip(
			old_in_blocks, new_in_blocks
		):
			new.weight[:out_old, new_start : new_start + old_size] = old.weight[
				:, old_start : old_start + old_size
			]
			if zero_new_inputs:
				new.weight[:out_old, new_start + old_size : new_start + new_size] = 0.0
		new.bias[:out_old] = old.bias


def blocks(*sizes):
	start = 0
	out = []
	for size in sizes:
		out.append((start, size))
		start += size
	return out


def main():
	parser = argparse.ArgumentParser(
		description="grow a nova net to more channels and rounds while it keeps computing the same policy"
	)
	parser.add_argument("source", help="checkpoint (.pt)")
	parser.add_argument("out", help="grown checkpoint (.pt)")
	parser.add_argument("--channels", type=int, required=True)
	parser.add_argument("--rounds", type=int, required=True)
	args = parser.parse_args()
	state = torch.load(args.source, map_location="cpu", weights_only=False)
	config = state["config"]
	old = Nova(config["channels"], config["rounds"])
	old.load_state_dict(state["model"])
	c0, c1 = config["channels"], args.channels
	new = Nova(c1, args.rounds)
	widen(
		old.embed,
		new.embed,
		blocks(NODE_FEATURES, GLOBAL_FEATURES),
		blocks(NODE_FEATURES, GLOBAL_FEATURES),
		zero_new_outputs=False,
	)
	for index, round_ in enumerate(new.rounds):
		if index < len(old.rounds):
			source = old.rounds[index]
			for name in ("own", "neighbors", "gather"):
				widen(
					getattr(source, name),
					getattr(round_, name),
					blocks(c0),
					blocks(c1),
					zero_new_outputs=False,
				)
			widen(
				source.triangle,
				round_.triangle,
				blocks(c0, TRIANGLE_FEATURES),
				[(0, c1), (c1, TRIANGLE_FEATURES)],
				zero_new_outputs=False,
			)
		else:
			with torch.no_grad():
				for name in ("own", "neighbors", "gather"):
					layer = getattr(round_, name)
					layer.weight.zero_()
					layer.bias.fill_(NEW_ROUND_BIAS)
				round_.triangle.weight.normal_(0.0, NOISE)
				round_.triangle.bias.zero_()
	for name in ("query", "key", "destination", "stay", "hold", "value_hidden"):
		source, target = getattr(old, name), getattr(new, name)
		if name == "value_hidden":
			widen(
				source,
				target,
				blocks(c0, c0, GLOBAL_FEATURES),
				[(0, c1), (c1, c1), (2 * c1, GLOBAL_FEATURES)],
				zero_new_outputs=True,
			)
		else:
			widen(source, target, blocks(c0), blocks(c1), zero_new_outputs=True)
	widen(
		old.triangle,
		new.triangle,
		blocks(c0, TRIANGLE_FEATURES),
		[(0, c1), (c1, TRIANGLE_FEATURES)],
		zero_new_outputs=False,
	)
	for name in ("spawn", "attack"):
		widen(
			getattr(old, name),
			getattr(new, name),
			blocks(c0, c0),
			[(0, c1), (c1, c1)],
			zero_new_outputs=True,
		)
	with torch.no_grad():
		new.value_out.weight.zero_()
		new.value_out.weight[:, : old.value_out.weight.shape[1]] = old.value_out.weight
		new.value_out.bias.copy_(old.value_out.bias)
	torch.save(
		{"model": new.state_dict(), "config": {"channels": c1, "rounds": args.rounds}},
		args.out,
	)


if __name__ == "__main__":
	main()
