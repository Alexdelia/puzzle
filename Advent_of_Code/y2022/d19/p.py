#!/usr/bin/env python3

import re
from collections import namedtuple
from enum import Enum
from os.path import dirname
from typing import Dict, List, NamedTuple, Tuple

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

DATA = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.\nBlueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian."

lines = DATA.splitlines()

class OreType(Enum):
    ORE = 0
    CLAY = 1
    OBSIDIAN = 2
    GEODE = 3

Ores = NamedTuple("Ores", [
    (OreType.ORE.name.lower()[0], int),
    (OreType.CLAY.name.lower()[0], int),
    (OreType.OBSIDIAN.name.lower()[1], int),
    (OreType.GEODE.name.lower()[0], int)])

Blueprint = Dict[OreType, Ores]

bs: List[Blueprint] = []

for l in lines:
    b: Blueprint = {}
    for r in re.finditer(r"Each (\w+) robot costs (\d+ \w+).*?(\d+ \w+)?\.()()", l):
        robot, *ores = r.groups()
        b[OreType[robot.upper()]] = Ores(*[int(o.split()[0]) if o else 0 for o in ores])
    bs.append(b)


class Game:
    def __init__(self, time: int, b: Blueprint):
        self.time = time
        self.b = b
        self.r = Ores(1, 0, 0, 0)
        self.o = Ores(0, 0, 0, 0)

    def solve(self) -> int:
        for _ in range(self.time):
            self.play()
        return self.o.g
    
    def play(self):
        self.o: Ores = Ores(*(self.o[i] + self.r[i] for i in range(len(self.o))))
        for robot in reversed(OreType):
            needed = self.b[robot]
            if all(self.o[i] >= needed[i] for i in range(len(self.o))):
                self.o = Ores(*(self.o[i] - needed[i] for i in range(len(self.o))))
                self.r = Ores(*(self.r[i] + 1 if robot == OreType(i) else self.r[i] for i in range(len(self.r))))
                break

t = 0
for i, g in enumerate([Game(24, b) for b in bs]):
    n = g.solve()
    print(f"blueprint {i + 1}:\t{n}")
    t += (i + 1) * n
print(f"part 1:\t{t}")
# print(f"part 1:\t{sum([(i + 1) * g.solve() for i, g in enumerate([Game(24, b) for b in bs])])}")
