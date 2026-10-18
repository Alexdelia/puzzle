import glob
import os
import sys

import numpy as np
from scipy.optimize import minimize

NAMES = [
	"bias",
	"cost",
	"town_region",
	"inst",
	"own_adj",
	"foe_adj",
	"town_adj",
	"degree_adj",
	"delta",
	"new_pairs",
	"rect",
	"aligned",
	"plan_mine",
	"plan_mult",
	"plan_foe",
	"route_close",
	"dist_own",
	"dist_town",
	"turn",
	"live_adj",
	"edge",
	"region_size",
	"region_foe",
	"region_own",
]
NF = len(NAMES)


def load(folder):
	games = []
	for feats in sorted(glob.glob(os.path.join(folder, "*.feats.tsv"))):
		labels = feats.replace(".feats.tsv", ".labels.tsv")
		chosen = set()
		for line in open(labels):
			turn, x, y = line.split()
			chosen.add((int(turn), int(x), int(y)))
		turns = {}
		for line in open(feats):
			parts = line.split("\t")
			turn, x, y = int(parts[0]), int(parts[1]), int(parts[2])
			values = np.array([float(v) for v in parts[3:]])
			turns.setdefault(turn, []).append(((turn, x, y) in chosen, values))
		groups = []
		for turn in sorted(turns):
			rows = turns[turn]
			labels_here = np.array([1.0 if picked else 0.0 for picked, _ in rows])
			if labels_here.sum() == 0:
				continue
			groups.append((np.stack([v for _, v in rows]), labels_here))
		games.append((os.path.basename(feats), groups))
	return games


def standardize(games):
	stack = np.concatenate([x for _, groups in games for x, _ in groups])
	mean = stack.mean(axis=0)
	std = stack.std(axis=0)
	std[std < 1e-9] = 1.0
	mean[0] = 0.0
	std[0] = 1.0
	return mean, std


def loss_and_grad(w, groups, mean, std, l2):
	total = 0.0
	grad = np.zeros_like(w)
	count = 0
	for x, y in groups:
		z = (x - mean) / std
		s = z @ w
		s -= s.max()
		p = np.exp(s)
		p /= p.sum()
		k = y.sum()
		total -= (y * np.log(p + 1e-12)).sum()
		grad -= z.T @ y
		grad += k * (z.T @ p)
		count += k
	total += l2 * (w[1:] ** 2).sum()
	grad[1:] += 2 * l2 * w[1:]
	return total / max(count, 1), grad / max(count, 1)


def fit(groups, mean, std, l2):
	w0 = np.zeros(NF)
	res = minimize(
		loss_and_grad,
		w0,
		args=(groups, mean, std, l2),
		jac=True,
		method="L-BFGS-B",
		options={"maxiter": 500},
	)
	return res.x


def evaluate(w, groups, mean, std):
	top1 = 0
	top3_hits = 0
	top3_total = 0
	for x, y in groups:
		s = ((x - mean) / std) @ w
		order = np.argsort(-s)
		top1 += y[order[0]]
		k = int(y.sum())
		top3_hits += y[order[:3]].sum()
		top3_total += min(k, 3)
	return top1 / len(groups), top3_hits / max(top3_total, 1)


def main():
	folder = sys.argv[1]
	l2 = float(sys.argv[2]) if len(sys.argv) > 2 else 0.01
	games = load(folder)
	mean, std = standardize(games)
	all_groups = [g for _, groups in games for g in groups]
	print(
		f"{len(games)} games, {len(all_groups)} turns, {sum(int(y.sum()) for _, y in all_groups)} placements"
	)

	held = []
	for fold in range(len(games)):
		train = [g for i, (_, groups) in enumerate(games) if i != fold for g in groups]
		test = games[fold][1]
		if not test:
			continue
		w = fit(train, mean, std, l2)
		top1, top3 = evaluate(w, test, mean, std)
		held.append((top1, top3, len(test)))
		print(
			f"  holdout {games[fold][0][:40]:<40} top1 {top1:.2f} top3 {top3:.2f} ({len(test)} turns)"
		)
	turns = sum(n for _, _, n in held)
	print(
		f"holdout pooled top1 {sum(a * n for a, _, n in held) / turns:.3f} top3 {sum(b * n for _, b, n in held) / turns:.3f}"
	)

	w = fit(all_groups, mean, std, l2)
	top1, top3 = evaluate(w, all_groups, mean, std)
	print(f"train top1 {top1:.3f} top3 {top3:.3f}")
	raw = w / std
	raw[0] = w[0] - (w[1:] * mean[1:] / std[1:]).sum()
	for name, value, scaled in zip(NAMES, raw, w):
		print(f"  {name:<12} {value:>10.4f}   (std-scaled {scaled:>8.3f})")
	print("const POLICY: [f64; NF] = [")
	print("\t" + ", ".join(f"{v:.5f}" for v in raw) + ",")
	print("];")


main()
