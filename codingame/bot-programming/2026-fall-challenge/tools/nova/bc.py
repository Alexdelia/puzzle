import argparse
import pathlib
import random
import sys
import zlib

import numpy as np
import torch

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from model import RECORD, Nova, expand, export, losses, tensors

TOP = (
	"Waffle3z",
	"AllanB",
	"putibuzu",
	"Pduhard-",
	"khanh-alice",
	"laconic_pixel",
	"tarriel",
	"unamed_player_",
	"kovi",
)


def player_of(path):
	return path.stem.split("-", 1)[1]


def held_out(path, share):
	return player_of(path) in TOP and zlib.crc32(path.stem.encode()) % 100 < share * 100


def load(paths):
	parts = [np.fromfile(p, dtype=RECORD) for p in paths]
	return np.concatenate([p for p in parts if len(p)])


def batches(data, size, rng):
	order = list(range(len(data["nodes"])))
	rng.shuffle(order)
	for start in range(0, len(order), size):
		index = torch.tensor(order[start : start + size])
		yield {key: value[index] for key, value in data.items()}


def to_device(batch, device):
	return expand(
		{key: value.to(device, non_blocking=True) for key, value in batch.items()}
	)


def move_agreement(move, batch):
	flows = batch["flows"]
	units = flows.sum(2, keepdim=True)
	probs = torch.softmax(move, 2)
	exact = probs * units
	floor = exact.floor()
	left = (units.squeeze(2) - floor.sum(2)).long()
	rank = (exact - floor).argsort(2, descending=True).argsort(2)
	predicted = floor + (rank < left[..., None]).float()
	eye = torch.eye(move.shape[1], dtype=torch.bool, device=move.device)
	off = ~eye
	actual = flows * off
	predicted = predicted * off
	return torch.minimum(actual, predicted).sum().item(), torch.maximum(
		actual, predicted
	).sum().item()


def evaluate(model, data, device, size=256):
	model.eval()
	move_loss_sum = triangle_loss_sum = positions = inter = union = hits = (
		labelled_total
	) = 0.0
	with torch.no_grad():
		for batch in batches(data, size, random.Random(0)):
			batch = to_device(batch, device)
			move_loss, triangle_loss, move, triangle = losses(model, batch)
			batch_inter, batch_union = move_agreement(move, batch)
			labelled = batch["action"] >= 0
			move_loss_sum += move_loss.item() * len(move)
			triangle_loss_sum += triangle_loss.item() * len(move)
			positions += len(move)
			inter += batch_inter
			union += batch_union
			hits += (triangle.argmax(2) == batch["action"])[labelled].sum().item()
			labelled_total += labelled.sum().item()
	model.train()
	return {
		"move_loss": move_loss_sum / positions,
		"triangle_loss": triangle_loss_sum / positions,
		"move_jaccard": inter / max(union, 1),
		"triangle_accuracy": hits / max(labelled_total, 1),
	}


def main():
	parser = argparse.ArgumentParser(
		description="behaviour-clone top players' moves into a nova net"
	)
	parser.add_argument(
		"data", nargs="+", help="directories of <game>-<nick>.bin dumps"
	)
	parser.add_argument(
		"--out",
		required=True,
		help="checkpoint path (.pt); the exported weights go next to it as .bin",
	)
	parser.add_argument("--epochs", type=int, default=40)
	parser.add_argument("--batch", type=int, default=64)
	parser.add_argument("--lr", type=float, default=2e-3)
	parser.add_argument("--channels", type=int, default=32)
	parser.add_argument("--rounds", type=int, default=3)
	parser.add_argument("--holdout", type=float, default=0.2)
	parser.add_argument(
		"--top-weight",
		type=int,
		default=1,
		help="repeat top players' games this many times",
	)
	parser.add_argument("--seed", type=int, default=1)
	parser.add_argument(
		"--top",
		default=None,
		help="comma list of players held out and weighted, replacing TOP",
	)
	parser.add_argument("--init", default=None, help="checkpoint (.pt) to fine-tune")
	parser.add_argument(
		"--train-only",
		default=None,
		help="train only on games whose player name contains this text",
	)
	args = parser.parse_args()
	if args.top:
		global TOP
		TOP = tuple(args.top.split(","))
	torch.manual_seed(args.seed)
	rng = random.Random(args.seed)
	paths = sorted(
		p for directory in args.data for p in pathlib.Path(directory).glob("*.bin")
	)
	test_paths = [p for p in paths if held_out(p, args.holdout)]
	train_paths = [
		p
		for p in paths
		if not held_out(p, args.holdout)
		and (args.train_only is None or args.train_only in player_of(p))
	]
	train_paths += [p for p in train_paths if player_of(p) in TOP] * (
		args.top_weight - 1
	)
	train_records, test_records = load(train_paths), load(test_paths)
	live_limit = int(max(train_records["live"].max(), test_records["live"].max()))
	print(
		f"train {len(train_records)} records from {len(train_paths)} games, test {len(test_records)} from {len(test_paths)}, live <= {live_limit}",
		flush=True,
	)
	device = "cuda" if torch.cuda.is_available() else "cpu"
	train, test = tensors(train_records, live_limit), tensors(test_records, live_limit)
	model = Nova(args.channels, args.rounds).to(device)
	if args.init:
		model.load_state_dict(
			torch.load(args.init, weights_only=False)["model"], strict=False
		)
	optimiser = torch.optim.AdamW(model.parameters(), lr=args.lr, weight_decay=1e-4)
	steps = args.epochs * ((len(train_records) + args.batch - 1) // args.batch)
	schedule = torch.optim.lr_scheduler.OneCycleLR(
		optimiser, max_lr=args.lr, total_steps=steps
	)
	best = None
	for epoch in range(args.epochs):
		for batch in batches(train, args.batch, rng):
			batch = to_device(batch, device)
			move_loss, triangle_loss, _, _ = losses(model, batch)
			loss = move_loss + triangle_loss
			optimiser.zero_grad()
			loss.backward()
			torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
			optimiser.step()
			schedule.step()
		metrics = evaluate(model, test, device)
		shown = " ".join(f"{k} {v:.4f}" for k, v in metrics.items())
		print(f"epoch {epoch + 1} {shown}", flush=True)
		if best is None or metrics["move_jaccard"] > best:
			best = metrics["move_jaccard"]
			torch.save(
				{
					"model": model.state_dict(),
					"config": {"channels": args.channels, "rounds": args.rounds},
				},
				args.out,
			)
			pathlib.Path(args.out).with_suffix(".bin").write_bytes(export(model))
	last = pathlib.Path(args.out).with_suffix(".last.pt")
	torch.save(
		{
			"model": model.state_dict(),
			"config": {"channels": args.channels, "rounds": args.rounds},
		},
		last,
	)
	last.with_suffix(".bin").write_bytes(export(model))
	print(f"best held-out move jaccard {best:.4f}")


if __name__ == "__main__":
	main()
