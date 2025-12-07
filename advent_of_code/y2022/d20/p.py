#!/usr/bin/env python3

from __future__ import annotations

import re
from os.path import dirname
from typing import Optional

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

DATA_EXAMPLE = "1\n2\n-3\n3\n-2\n0\n4"


class Node:
	def __init__(
		self, n: int, prev: Optional[Node] = None, next: Optional[Node] = None
	):
		self.n = n
		self.prev = prev
		self.next = next


def parse(data: str, key: int = 1) -> list[Node]:
	lines = data.splitlines()
	sequence: list[Node] = [Node(int(l) * key) for l in lines]

	for a, b in zip(sequence, sequence[1:]):
		a.next = b
		b.prev = a

	sequence[0].prev = sequence[-1]
	sequence[-1].next = sequence[0]

	return sequence


def solve(sequence: list[Node], iteration: int) -> tuple[int, int, int]:
	for _ in range(iteration):
		for node in sequence:
			node.prev.next = node.next
			node.next.prev = node.prev

			a, b = node.prev, node.next

			for _ in range(node.n % (len(sequence) - 1)):
				a = a.next
				b = b.next

			a.next = node
			node.prev = a
			b.prev = node
			node.next = b

	ret: list[int] = []
	for node in sequence:
		if node.n == 0:
			t = node
			for _ in range(3):
				for _ in range(1000):
					t = t.next
				ret.append(t.n)
			break

	return tuple(ret)


sequence = parse(DATA_EXAMPLE)
n = solve(sequence, 1)
assert n == (4, -3, 2), f"Expected (4, -3, 2), got {n}"
assert sum(n) == 3, f"Expected 3, got {sum(n)}"

sequence = parse(DATA)
n = solve(sequence, 1)
print(f"part 1:\t{sum(n)}")

KEY = 811589153
sequence = parse(DATA_EXAMPLE, KEY)
n = solve(sequence, 10)
assert n == (811589153, 2434767459, -1623178306), (
	f"Expected (811589153, 2434767459, 1623178306), got {n}"
)
assert sum(n) == 1623178306, f"Expected 1623178306, got {sum(n)}"

sequence = parse(DATA, KEY)
n = solve(sequence, 10)
print(f"part 2:\t{sum(n)}")
