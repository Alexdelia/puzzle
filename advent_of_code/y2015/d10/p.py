#!/usr/bin/env python3

import sys

PART_1_ITERATION_COUNT = 40
PART_2_ITERATION_COUNT = 50


def look_and_say(s: str) -> str:
	res = ""

	i = 0
	while i < len(s):
		count = 0
		c = s[i]

		while i < len(s) and s[i] == c:
			count += 1
			i += 1

		res += str(count) + c

	return res


def solve(data: str) -> tuple[int, int]:
	s = data.strip()
	print(f"{0}:\t{len(s)}")

	for i in range(max(PART_1_ITERATION_COUNT, PART_2_ITERATION_COUNT)):
		s = look_and_say(s)
		print(f"{i + 1}:\t{len(s)}")

		if i + 1 == PART_1_ITERATION_COUNT:
			p1 = len(s)

	print()

	p2 = len(s)

	return (p1, p2)


test = [
	("1", "11"),
	("11", "21"),
	("21", "1211"),
	("1211", "111221"),
	("111221", "312211"),
]
for t in test:
	got = look_and_say(t[0])
	assert got == t[1], f"look_and_say({t[0]}) = {got}, expected {t[1]}"


sys.path.append("../..")
from get_data import get_data

p1, p2 = solve(get_data())
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
