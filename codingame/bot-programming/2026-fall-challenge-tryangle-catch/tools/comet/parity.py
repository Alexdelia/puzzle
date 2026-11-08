import argparse
import sys

import numpy as np
import torch

sys.path.insert(0, __import__("os").path.dirname(__file__))
from train import RECORD, Net, batch_tensors


def main():
	parser = argparse.ArgumentParser(
		description="print the torch value of every record of one stream"
	)
	parser.add_argument("record")
	parser.add_argument("weights")
	parser.add_argument("--width", type=int, default=32)
	parser.add_argument("--depth", type=int, default=3)
	args = parser.parse_args()
	raw = np.fromfile(args.record, dtype=RECORD)
	tri = raw["triangles"]
	label = tri["label"].astype(np.int64)
	data = {
		"nodes": raw["nodes"].astype(np.float16),
		"globals": raw["globals"],
		"neighbors": raw["neighbors"],
		"units": raw["units"],
		"moves": raw["moves"],
		"corners": tri["corners"].astype(np.int16),
		"labels": label.astype(np.int16),
		"masks": tri["mask"].view(np.int64),
		"value": np.zeros(len(raw), np.float32),
		"count": raw["count"],
	}
	net = Net(args.width, args.depth)
	net.load_state_dict(torch.load(args.weights, map_location="cpu"))
	net.eval()
	for k, v in net.state_dict().items():
		v.copy_(v.half().float())
	with torch.no_grad():
		t = batch_tensors(data, np.arange(len(raw)), "cpu")
		h = net.trunk(t["nodes"], t["globals"], t["neighbors"], t["valid"])
		values = net.value(h, t["globals"], t["valid"])
	for turn, value in zip(raw["turn"], values.tolist()):
		print(f"turn {turn} value {value:.5f}")


if __name__ == "__main__":
	main()
