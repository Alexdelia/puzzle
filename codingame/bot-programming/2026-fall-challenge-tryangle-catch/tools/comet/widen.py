import argparse

import torch

from train import GLOBAL_FEATURES, Net, export

HIDDEN_TO_HIDDEN = [
	"move_from",
	"move_to",
	"spawn_triangle",
	"spawn_corner",
	"attack_triangle",
	"attack_target",
]
OUTPUT_VECTORS = ["move_stay", "move_out", "spawn_out", "attack_out"]


def hidden_columns(narrow, wide, blocks, extra=0):
	parts = [torch.arange(narrow) + block * wide for block in range(blocks)]
	parts.append(torch.arange(extra) + blocks * wide)
	return torch.cat(parts)


def graft(source, target, rows, columns):
	with torch.no_grad():
		target.weight[rows] = 0
		target.weight[rows[:, None], columns[None, :]] = source.weight
		if source.bias is not None:
			target.bias[rows] = source.bias


def widen(narrow_net, wide):
	narrow = narrow_net.width
	depth = len(narrow_net.own)
	net = Net(wide, depth)
	hidden = torch.arange(narrow)
	single = torch.arange(1)
	with torch.no_grad():
		net.input.weight[hidden] = narrow_net.input.weight
		net.input.bias[hidden] = narrow_net.input.bias
	one_block = hidden_columns(narrow, wide, 1)
	for source, target in zip(narrow_net.own, net.own):
		graft(source, target, hidden, one_block)
	for source, target in zip(narrow_net.mean, net.mean):
		graft(source, target, hidden, one_block)
	for name in HIDDEN_TO_HIDDEN:
		graft(getattr(narrow_net, name), getattr(net, name), hidden, one_block)
	graft(narrow_net.triangle, net.triangle, hidden, hidden_columns(narrow, wide, 2))
	graft(narrow_net.hold, net.hold, single, one_block)
	graft(
		narrow_net.value_hidden,
		net.value_hidden,
		hidden,
		hidden_columns(narrow, wide, 2, GLOBAL_FEATURES),
	)
	graft(narrow_net.value_out, net.value_out, single, one_block)
	with torch.no_grad():
		for name in OUTPUT_VECTORS:
			vector = getattr(net, name)
			vector.zero_()
			vector[:narrow] = getattr(narrow_net, name)
	return net


def main():
	parser = argparse.ArgumentParser(
		description="widen a comet net without changing its outputs"
	)
	parser.add_argument("--init", required=True, help="state dict of the narrow net")
	parser.add_argument("--width", type=int, required=True)
	parser.add_argument("--depth", type=int, default=3)
	parser.add_argument("--narrow-width", type=int, default=32)
	parser.add_argument("--out", required=True)
	args = parser.parse_args()
	narrow_net = Net(args.narrow_width, args.depth)
	narrow_net.load_state_dict(torch.load(args.init, map_location="cpu"))
	net = widen(narrow_net, args.width)
	torch.save(net.state_dict(), args.out + ".pt")
	export(net, args.out)


if __name__ == "__main__":
	main()
