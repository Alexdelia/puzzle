#!/usr/bin/env python3

import re
from enum import Enum
from itertools import combinations_with_replacement
from os.path import dirname
from typing import NamedTuple

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

DATA = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.\nBlueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian."

lines = DATA.splitlines()


class OreType(Enum):
	ORE = 0
	CLAY = 1
	OBSIDIAN = 2
	GEODE = 3


TOres = NamedTuple(
	"TOres",
	[
		(OreType.ORE.name.lower()[0], int),
		(OreType.CLAY.name.lower()[0], int),
		(OreType.OBSIDIAN.name.lower()[1], int),
		(OreType.GEODE.name.lower()[0], int),
	],
)

LOres = list[int]

Blueprint = tuple[TOres, TOres, TOres, TOres]

bs: list[Blueprint] = []

for l in lines:
	b: list[TOres] = []
	for r in re.finditer(r"Each (\w+) robot costs (\d+ \w+).*?(\d+ \w+)?\.", l):
		robot, *ores = r.groups()
		sorted_ore: list[int] = [0, 0, 0, 0]
		for o in ores:
			if o:
				n, name = o.split()
				sorted_ore[OreType[name.upper()].value] = int(n)
		b.append(TOres(*sorted_ore))
	bs.append(tuple(b))


class Game:
	def __init__(self, time: int, b: Blueprint, actions: tuple[int, ...]):
		self.time: int = time
		self.b: Blueprint = b
		self.r: LOres = [1, 0, 0, 0]
		self.o: LOres = [0, 0, 0, 0]
		self.actions: tuple[int, ...] = actions

	def solve(self) -> int:
		while self.time > 0:
			self.time -= 1
			self.play()
		# print("ores", self.o)
		# print("robot", self.r)
		return self.o[OreType.GEODE.value]

	def play(self):
		to_build = -1
		robot = self.actions[self.time - 1]
		needed = self.b[robot]
		if all(self.o[i] >= needed[i] for i in range(len(self.o))):
			# print(f"robot {robot}:\t", needed)
			for i in range(len(self.o)):
				self.o[i] -= needed[i]
				to_build = robot

		# print("produce:\t", self.r)
		for i in range(len(self.r)):
			self.o[i] += self.r[i]
		# print("ores:\t\t", self.o)

		if to_build != -1:
			self.r[to_build] += 1


t = 0
for bn in bs:
	m = 0
	for a in combinations_with_replacement(range(4), 24):
		g = Game(24, bn, a)
		n = g.solve()
		if n > m:
			m = n
	t += m
print(f"part 1:\t{t}")
# print(f"part 1:\t{sum([(i + 1) * g.solve() for i, g in enumerate([Game(24, b) for b in bs])])}")
