#!/usr/bin/env python3

import sys

sys.path.append("../..")
from get_data import get_data

DATA: str = get_data()

range_chunk, item_chunk = DATA.split("\n\n")

ranges = [tuple(map(int, line.split("-"))) for line in range_chunk.splitlines()]
items = [int(line) for line in item_chunk.splitlines()]


def merge_ranges(ranges: list[tuple[int, int]]) -> list[tuple[int, int]]:
	ranges = sorted(ranges, key=lambda x: x[0])
	merged = [ranges.pop(0)]

	for r in ranges:
		if r[0] <= merged[-1][1]:
			merged[-1] = (merged[-1][0], max(r[1], merged[-1][1]))
			continue

		merged.append(r)

	return merged


ranges = merge_ranges(ranges)


p1 = 0
p2 = 0

for r in ranges:
	for i in range(len(items) - 1, -1, -1):
		if r[0] <= items[i] <= r[1]:
			p1 += 1
			items.pop(i)

	p2 += r[1] - r[0] + 1

print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
