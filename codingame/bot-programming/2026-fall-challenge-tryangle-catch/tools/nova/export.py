import argparse
import pathlib
import sys

import torch

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from model import Nova, export


def main():
	parser = argparse.ArgumentParser(
		description="export a nova checkpoint (.pt) to the weights file the Rust bot reads"
	)
	parser.add_argument("checkpoint")
	parser.add_argument("out")
	args = parser.parse_args()
	state = torch.load(args.checkpoint, map_location="cpu", weights_only=False)
	model = Nova(**state["config"])
	model.load_state_dict(state["model"])
	pathlib.Path(args.out).write_bytes(export(model))


if __name__ == "__main__":
	main()
