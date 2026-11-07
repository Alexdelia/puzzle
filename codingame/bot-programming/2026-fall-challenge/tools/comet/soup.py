import argparse
import sys

import torch

sys.path.insert(0, __import__("os").path.dirname(__file__))
from train import Net, export


def main():
	parser = argparse.ArgumentParser(
		description="average the weights of several comet nets and export the result"
	)
	parser.add_argument("nets", nargs="+")
	parser.add_argument("--out", required=True)
	parser.add_argument("--width", type=int, default=32)
	parser.add_argument("--depth", type=int, default=3)
	args = parser.parse_args()
	states = [torch.load(path, map_location="cpu") for path in args.nets]
	averaged = {k: sum(s[k] for s in states) / len(states) for k in states[0]}
	net = Net(args.width, args.depth)
	net.load_state_dict(averaged)
	torch.save(averaged, args.out + ".pt")
	export(net, args.out)


if __name__ == "__main__":
	main()
