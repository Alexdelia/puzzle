import random
import sys

FEATS = 11
NAMES = [
	"bias",
	"fmult",
	"live",
	"cost",
	"town",
	"inst",
	"horizon",
	"touch",
	"foes",
	"fmult*horizon",
	"fmult*town",
]


def load(path):
	rows = []
	for line in open(path):
		value = [float(field) for field in line.split("\t")]
		rows.append((value[0], value[1:]))
	return rows


def solve(data, ridge):
	table = [[0.0] * (FEATS + 1) for _ in range(FEATS)]
	for label, feat in data:
		for i in range(FEATS):
			if feat[i] == 0.0:
				continue
			row = table[i]
			for j in range(FEATS):
				row[j] += feat[i] * feat[j]
			row[FEATS] += feat[i] * label
	for i in range(FEATS):
		table[i][i] += ridge
	for col in range(FEATS):
		pivot = max(range(col, FEATS), key=lambda r: abs(table[r][col]))
		table[col], table[pivot] = table[pivot], table[col]
		scale = table[col][col]
		for j in range(col, FEATS + 1):
			table[col][j] /= scale
		for r in range(FEATS):
			if r == col or table[r][col] == 0.0:
				continue
			factor = table[r][col]
			for j in range(col, FEATS + 1):
				table[r][j] -= factor * table[col][j]
	return [table[i][FEATS] for i in range(FEATS)]


def predict(weights, feat):
	return max(0.0, sum(w * x for w, x in zip(weights, feat)))


def r2(data, weights):
	labels = [label for label, _ in data]
	mean = sum(labels) / len(labels)
	total = sum((label - mean) ** 2 for label in labels)
	left = sum((label - predict(weights, feat)) ** 2 for label, feat in data)
	return 1 - left / total


def quartiles(data, key):
	order = sorted(range(len(data)), key=lambda i: key(data[i][1]))
	labels = [data[i][0] for i in order]
	quarter = len(labels) // 4
	return sum(labels[:quarter]) / quarter, sum(labels[-quarter:]) / quarter


def main():
	rows = load(sys.argv[1])
	random.seed(7)
	random.shuffle(rows)
	cut = int(len(rows) * 0.8)
	train, test = rows[:cut], rows[cut:]
	weights = solve(train, 1e-6)

	print(
		f"samples {len(rows)}  dead {sum(1 for l, _ in rows if l == 0) / len(rows):.1%}"
	)
	for name, weight in zip(NAMES, weights):
		print(f"  {name:<14} {weight:+.4f}")
	print(f"R2 train {r2(train, weights):.4f}  test {r2(test, weights):.4f}")

	keys = {
		"mult / cost": lambda f: (1.5 if f[4] else 1.0) * f[1] / f[3],
		"value / cost": lambda f: predict(weights, f) / f[3],
	}
	for name, key in keys.items():
		low, high = quartiles(test, key)
		print(
			f"{name:<14} bottom quartile pays {low:6.1f}  top quartile pays {high:6.1f}"
		)

	raw = sum(feat[1] for _, feat in rows) / len(rows)
	fitted = sum(predict(weights, feat) / (feat[6] * 100.0) for _, feat in rows) / len(
		rows
	)
	scale = raw / fitted
	print(f"scale {scale:.6f}")
	print("const LIN: [f64; FEATS] = [")
	for weight in weights:
		print(f"\t{weight * scale:.5f},")
	print("];")


main()
