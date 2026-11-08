import argparse
import os
import struct

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

NODES = 64
NODE_FEATURES = 29
GLOBAL_FEATURES = 8
MAX_DEGREE = 12
SLOTS = MAX_DEGREE + 1
MAX_USABLE = 32
ATTACK = 4
CLASSES = ATTACK + NODES
MAGIC = 0x3154_4D43

RECORD = np.dtype(
	[
		("turn", "<i4"),
		("count", "<i4"),
		("usable", "<i4"),
		("pad", "<i4"),
		("nodes", "<f4", (NODES, NODE_FEATURES)),
		("globals", "<f4", (GLOBAL_FEATURES,)),
		("neighbors", "i1", (NODES, MAX_DEGREE)),
		("units", "<i2", (NODES,)),
		("moves", "<i2", (NODES, SLOTS)),
		(
			"triangles",
			[
				("corners", "u1", (3,)),
				("label", "i1"),
				("target", "i1"),
				("pad", "u1", (3,)),
				("mask", "<u8"),
			],
			(MAX_USABLE,),
		),
	]
)
assert RECORD.itemsize == 10544


def load(record_dir, labels_path, cache):
	if os.path.exists(cache):
		data = np.load(cache)
		return {k: data[k] for k in data.files}
	outcome = {}
	for line in open(labels_path):
		name, value = line.rstrip("\n").split("\t")
		outcome[name] = float(value)
	parts = {
		k: []
		for k in [
			"nodes",
			"globals",
			"neighbors",
			"units",
			"moves",
			"corners",
			"labels",
			"masks",
			"value",
			"game",
			"turn",
			"count",
			"player",
		]
	}
	names = sorted(f[:-4] for f in os.listdir(record_dir) if f.endswith(".rec"))
	for name in names:
		raw = np.fromfile(os.path.join(record_dir, name + ".rec"), dtype=RECORD)
		if len(raw) == 0 or name not in outcome:
			continue
		game, player = name.split("-", 1)
		tri = raw["triangles"]
		label = tri["label"].astype(np.int64)
		target = tri["target"].astype(np.int64)
		cls = np.where(label == ATTACK, ATTACK + target, label)
		cls = np.where(label < 0, -1, cls)
		parts["nodes"].append(raw["nodes"].astype(np.float16))
		parts["globals"].append(raw["globals"])
		parts["neighbors"].append(raw["neighbors"])
		parts["units"].append(raw["units"])
		parts["moves"].append(raw["moves"])
		parts["corners"].append(tri["corners"].astype(np.int16))
		parts["labels"].append(cls.astype(np.int16))
		parts["masks"].append(tri["mask"].view(np.int64))
		parts["value"].append(np.full(len(raw), outcome[name], np.float32))
		parts["game"].append(np.full(len(raw), int(game), np.int64))
		parts["turn"].append(raw["turn"])
		parts["count"].append(raw["count"])
		parts["player"].append(np.array([player] * len(raw)))
	data = {k: np.concatenate(v) for k, v in parts.items()}
	np.savez(cache, **data)
	return data


class Net(nn.Module):
	def __init__(self, width, depth):
		super().__init__()
		w = width
		self.width = width
		self.input = nn.Linear(NODE_FEATURES + GLOBAL_FEATURES, w)
		self.own = nn.ModuleList([nn.Linear(w, w) for _ in range(depth)])
		self.mean = nn.ModuleList([nn.Linear(w, w, bias=False) for _ in range(depth)])
		self.move_from = nn.Linear(w, w)
		self.move_to = nn.Linear(w, w, bias=False)
		self.move_stay = nn.Parameter(torch.zeros(w))
		self.move_out = nn.Parameter(torch.randn(w) / w**0.5)
		self.triangle = nn.Linear(2 * w, w)
		self.hold = nn.Linear(w, 1)
		self.spawn_triangle = nn.Linear(w, w)
		self.spawn_corner = nn.Linear(w, w, bias=False)
		self.spawn_out = nn.Parameter(torch.randn(w) / w**0.5)
		self.attack_triangle = nn.Linear(w, w)
		self.attack_target = nn.Linear(w, w, bias=False)
		self.attack_out = nn.Parameter(torch.randn(w) / w**0.5)
		self.value_hidden = nn.Linear(2 * w + GLOBAL_FEATURES, w)
		self.value_out = nn.Linear(w, 1)

	def trunk(self, nodes, globals_, neighbors, valid):
		b, n, _ = nodes.shape
		x = torch.cat([nodes, globals_[:, None, :].expand(b, n, GLOBAL_FEATURES)], -1)
		h = F.relu(self.input(x)) * valid[..., None]
		nmask = (neighbors >= 0).float()
		index = neighbors.clamp(min=0).long()
		degree = nmask.sum(-1, keepdim=True).clamp(min=1)
		for own, mean in zip(self.own, self.mean):
			gathered = torch.gather(
				h[:, None, :, :].expand(b, n, n, self.width),
				2,
				index[..., None].expand(b, n, MAX_DEGREE, self.width),
			)
			pooled = (gathered * nmask[..., None]).sum(2) / degree
			h = (h + F.relu(own(h) + mean(pooled))) * valid[..., None]
		return h

	def value(self, h, globals_, valid):
		count = valid.sum(1, keepdim=True)
		mean = h.sum(1) / count
		big = torch.where(valid[..., None] > 0, h, torch.full_like(h, -1e9))
		top = big.max(1).values
		hidden = F.relu(self.value_hidden(torch.cat([mean, top, globals_], -1)))
		return self.value_out(hidden).squeeze(-1)

	def moves(self, h, neighbors):
		b, n, w = h.shape
		source = self.move_from(h)
		dest = self.move_to(h)
		index = neighbors.clamp(min=0).long()
		dest_n = torch.gather(
			dest[:, None, :, :].expand(b, n, n, w),
			2,
			index[..., None].expand(b, n, MAX_DEGREE, w),
		)
		stay = F.relu(source + dest + self.move_stay) @ self.move_out
		go = F.relu(source[:, :, None, :] + dest_n) @ self.move_out
		logits = torch.cat([stay[..., None], go], -1)
		mask = torch.cat(
			[torch.ones_like(neighbors[..., :1], dtype=torch.bool), neighbors >= 0], -1
		)
		return logits.masked_fill(~mask, -1e9)

	def actions(self, h, corners, masks):
		b, n, w = h.shape
		t = corners.shape[1]
		index = corners.clamp(min=0, max=n - 1).long()
		hc = torch.gather(
			h[:, None, :, :].expand(b, t, n, w), 2, index[..., None].expand(b, t, 3, w)
		)
		summary = F.relu(self.triangle(torch.cat([hc.mean(2), hc.max(2).values], -1)))
		hold = self.hold(summary)
		a = self.spawn_triangle(summary)
		spawn = F.relu(a[:, :, None, :] + self.spawn_corner(hc)) @ self.spawn_out
		a2 = self.attack_triangle(summary)
		target = self.attack_target(h)
		attack = F.relu(a2[:, :, None, :] + target[:, None, :, :]) @ self.attack_out
		bits = torch.arange(n, device=h.device)
		allowed = ((masks[..., None] >> bits) & 1).bool()
		attack = attack.masked_fill(~allowed, -1e9)
		return torch.cat([hold, spawn, attack], -1)


def export(net, path):
	w = net.width
	values = []

	def put(tensor):
		values.append(tensor.detach().float().cpu().reshape(-1).numpy())

	put(net.input.weight)
	put(net.input.bias)
	for own, mean in zip(net.own, net.mean):
		put(own.weight)
		put(own.bias)
		put(mean.weight)
	put(net.move_from.weight)
	put(net.move_from.bias)
	put(net.move_to.weight)
	put(net.move_stay)
	put(net.move_out)
	put(net.triangle.weight)
	put(net.triangle.bias)
	put(net.hold.weight)
	put(net.hold.bias)
	put(net.spawn_triangle.weight)
	put(net.spawn_triangle.bias)
	put(net.spawn_corner.weight)
	put(net.spawn_out)
	put(net.attack_triangle.weight)
	put(net.attack_triangle.bias)
	put(net.attack_target.weight)
	put(net.attack_out)
	put(net.value_hidden.weight)
	put(net.value_hidden.bias)
	put(net.value_out.weight)
	put(net.value_out.bias)
	flat = np.concatenate(values).astype("<f2")
	with open(path, "wb") as out:
		out.write(
			struct.pack("<5I", MAGIC, NODE_FEATURES, GLOBAL_FEATURES, w, len(net.own))
		)
		out.write(flat.tobytes())
	print(f"exported {flat.size} parameters to {path}")


def batch_tensors(data, idx, device):
	pick = lambda k: torch.from_numpy(np.ascontiguousarray(data[k][idx])).to(device)
	count = pick("count")
	nodes = data["nodes"].shape[1]
	valid = (torch.arange(nodes, device=device)[None, :] < count[:, None]).float()
	return {
		"nodes": pick("nodes").float(),
		"globals": pick("globals"),
		"neighbors": pick("neighbors").long(),
		"units": pick("units").float(),
		"moves": pick("moves").float(),
		"corners": pick("corners").long(),
		"labels": pick("labels").long(),
		"masks": pick("masks"),
		"value": pick("value"),
		"search": pick("search")
		if "search" in data
		else torch.full_like(count, float("nan"), dtype=torch.float32),
		"imitate": (pick("source") != 2).float()
		if "source" in data
		else torch.ones_like(count, dtype=torch.float32),
		"valid": valid,
	}


def losses(net, t, bootstrap=0.0):
	h = net.trunk(t["nodes"], t["globals"], t["neighbors"], t["valid"])
	value = net.value(h, t["globals"], t["valid"])
	target = (t["value"] + 1) / 2
	searched = torch.isfinite(t["search"])
	if bootstrap > 0 and searched.any():
		blend = (1 - bootstrap) * target + bootstrap * torch.sigmoid(
			torch.nan_to_num(t["search"])
		)
		target = torch.where(searched, blend, target)
	value_loss = F.binary_cross_entropy_with_logits(value, target)
	move_logits = net.moves(h, t["neighbors"])
	has = (t["units"] > 0) & (t["imitate"][:, None] > 0)
	target = t["moves"] / t["units"].clamp(min=1)[..., None]
	logp = F.log_softmax(move_logits, -1)
	per_node = -(target * logp).sum(-1)
	if has.any():
		move_loss = per_node[has].mean()
		move_acc = (move_logits.argmax(-1) == t["moves"].argmax(-1))[has].float().mean()
	else:
		move_loss = torch.zeros((), device=h.device)
		move_acc = torch.zeros((), device=h.device)
	action_logits = net.actions(h, t["corners"], t["masks"])
	labelled = (t["labels"] >= 0) & (t["imitate"][:, None] > 0)
	if labelled.any():
		action_loss = F.cross_entropy(action_logits[labelled], t["labels"][labelled])
		action_acc = (
			(action_logits[labelled].argmax(-1) == t["labels"][labelled]).float().mean()
		)
	else:
		action_loss = torch.zeros((), device=h.device)
		action_acc = torch.zeros((), device=h.device)
	return value_loss, move_loss, action_loss, move_acc, action_acc


def main():
	parser = argparse.ArgumentParser(
		description="train the comet graph net on extracted records"
	)
	parser.add_argument("--data", required=True, help="directory written by build.py")
	parser.add_argument(
		"--data2",
		nargs="*",
		default=[],
		help="further build.py directories mixed into every batch",
	)
	parser.add_argument(
		"--mix",
		type=float,
		nargs="*",
		default=[],
		help="share of each batch drawn from each --data2 directory",
	)
	parser.add_argument("--policy-weight", type=float, default=1.0)
	parser.add_argument("--out", required=True)
	parser.add_argument("--width", type=int, default=32)
	parser.add_argument("--depth", type=int, default=3)
	parser.add_argument("--epochs", type=int, default=6)
	parser.add_argument("--batch", type=int, default=512)
	parser.add_argument("--lr", type=float, default=2e-3)
	parser.add_argument("--value-weight", type=float, default=0.5)
	parser.add_argument("--init")
	parser.add_argument(
		"--bootstrap",
		type=float,
		default=0.0,
		help="share of the value target taken from the recorded search score",
	)
	args = parser.parse_args()
	device = "cuda"

	def open_pack(path):
		return {
			f[:-4]: np.load(os.path.join(path, f), mmap_mode="r")
			for f in os.listdir(path)
			if f.endswith(".npy")
		}

	def split(pack):
		holdout = (pack["game"] % 20) == 0
		return np.flatnonzero(~holdout), np.flatnonzero(holdout)

	data = open_pack(args.data)
	train_idx, test_idx = split(data)
	print(f"records {len(data['game'])} train {len(train_idx)} holdout {len(test_idx)}")
	assert len(args.mix) == len(args.data2)
	extras = []
	for path, share in zip(args.data2, args.mix):
		pack = open_pack(path)
		train2, test2 = split(pack)
		print(
			f"{path}: records {len(pack['game'])} train {len(train2)} holdout {len(test2)}"
		)
		extras.append((path, pack, train2, test2, int(args.batch * share)))
	first = args.batch - sum(extra[4] for extra in extras)
	net = Net(args.width, args.depth).to(device)
	if args.init:
		net.load_state_dict(torch.load(args.init))
	opt = torch.optim.Adam(net.parameters(), lr=args.lr)
	steps = args.epochs * (len(train_idx) // first)
	sched = torch.optim.lr_scheduler.OneCycleLR(opt, max_lr=args.lr, total_steps=steps)
	rng = np.random.default_rng(1)
	for epoch in range(args.epochs):
		net.train()
		order = rng.permutation(train_idx)
		totals = np.zeros(5)
		batches = 0
		for start in range(0, len(order) - first + 1, first):
			t = batch_tensors(data, np.sort(order[start : start + first]), device)
			for _, pack, train2, _, size in extras:
				t2 = batch_tensors(pack, np.sort(rng.choice(train2, size)), device)
				t = {k: torch.cat([t[k], t2[k]]) for k in t}
			vl, ml, al, ma, aa = losses(net, t, args.bootstrap)
			loss = args.policy_weight * (ml + al) + args.value_weight * vl
			opt.zero_grad()
			loss.backward()
			opt.step()
			sched.step()
			totals += [vl.item(), ml.item(), al.item(), ma.item(), aa.item()]
			batches += 1
		net.eval()
		tests = np.zeros(5)
		tb = 0
		with torch.no_grad():
			for start in range(0, len(test_idx), 2048):
				t = batch_tensors(data, test_idx[start : start + 2048], device)
				tests += [x.item() for x in losses(net, t)]
				tb += 1
			for path, pack, _, test2, _ in extras:
				tests2 = np.zeros(5)
				tb2 = 0
				for start in range(0, len(test2), 2048):
					t = batch_tensors(pack, test2[start : start + 2048], device)
					tests2 += [x.item() for x in losses(net, t)]
					tb2 += 1
				te2 = tests2 / max(tb2, 1)
				print(
					f"epoch {epoch} {os.path.basename(path)} holdout value {te2[0]:.3f} move {te2[1]:.3f} action {te2[2]:.3f} macc {te2[3]:.3f} aacc {te2[4]:.3f}",
					flush=True,
				)
		tr = totals / batches
		te = tests / tb
		print(
			f"epoch {epoch} train value {tr[0]:.3f} move {tr[1]:.3f} action {tr[2]:.3f} macc {tr[3]:.3f} aacc {tr[4]:.3f} | holdout value {te[0]:.3f} move {te[1]:.3f} action {te[2]:.3f} macc {te[3]:.3f} aacc {te[4]:.3f}",
			flush=True,
		)
		torch.save(net.state_dict(), args.out + ".pt")
		export(net, args.out)


if __name__ == "__main__":
	main()
