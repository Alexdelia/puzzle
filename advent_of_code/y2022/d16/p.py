#!/usr/bin/env python3

import heapq
import re
from os.path import dirname
from typing import Dict, List, Set, Tuple, Union

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

d: Dict[str, Tuple[int, List[str]]] = {}
pressure: Set[str] = set()

for l in lines:
	s = re.sub("[(rate=};,]", "", l).split()
	d[s[1]] = (int(s[4]), s[9:])
	if d[s[1]][0] > 0:
		pressure.add(s[1])


class Node:
	def __init__(self, valve: str, time: int, released: int, visited: Set[str]):
		self.valve = valve
		self.time = time
		self.released = released
		self.visited = visited
		self.all_visited: bool = pressure.issubset(visited)

	def __lt__(self, other):
		return self.priority() > other.priority()

	def __repr__(self):
		return f"Node({self.valve}, {self.time}, {self.released}, {self.visited})"

	def priority(self) -> int:
		return (
			int(self.valve in self.visited) * -100_000
			+ self.released
			+ (30 - self.time) * 100_000
			+ int(self.all_visited) * 100_000_000
		)

	def explore(self):
		if self.valve not in self.visited:
			self.released += self.time * d[self.valve][0]
			self.visited.add(self.valve)
			self.time -= 1
		self.time -= 1


open = [Node("AA", 30, 0, set())]
heapq.heapify(open)
max_realease = 0

i = 0
while open:
	if i % 100_000 == 0:
		print(f"{i}\t{len(open)}", end="\r")
	i += 1
	i %= 100_000_000

	node = heapq.heappop(open)
	node.explore()

	if node.time <= 0 or node.all_visited:
		if node.released > max_realease:
			max_realease = node.released
			print(f"\nnew max:\t{max_realease}")
		continue

	for v in d[node.valve][1]:
		heapq.heappush(open, Node(v, node.time, node.released, node.visited.copy()))

print(f"part 1:\t{max_realease}")
