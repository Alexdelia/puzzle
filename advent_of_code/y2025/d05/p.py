#!/usr/bin/env python3

import sys

sys.path.append("../..")
from get_data import get_data

DATA: str = get_data()

range_chunk, item_chunk = DATA.split("\n\n")

ranges = [tuple(map(int, line.split("-"))) for line in range_chunk.splitlines()]
items = [int(line) for line in item_chunk.splitlines()]

p1 = 0

for r in ranges:
	for i in range(len(items) - 1, -1, -1):
		if r[0] <= items[i] <= r[1]:
			p1 += 1
			items.pop(i)

print(f"part 1:\t{p1}")


p2 = 0


print(f"part 2:\t{p2}")
