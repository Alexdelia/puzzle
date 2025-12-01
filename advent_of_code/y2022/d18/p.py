#!/usr/bin/env python3

import re
from collections import namedtuple
from os.path import dirname
from typing import FrozenSet, List, Set

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

# DATA = "2,2,2\n1,2,2\n3,2,2\n2,1,2\n2,3,2\n2,2,1\n2,2,3\n2,2,4\n2,2,6\n1,2,5\n3,2,5\n2,1,5\n2,3,5"

lines = DATA.splitlines()

Cube = namedtuple("Cube", "x y z")

cubes: FrozenSet[Cube] = frozenset([Cube(*map(int, l.split(","))) for l in lines])


def sides(c: Cube) -> FrozenSet[Cube]:
    return frozenset(
        [
            Cube(c.x + 1, c.y, c.z),
            Cube(c.x - 1, c.y, c.z),
            Cube(c.x, c.y + 1, c.z),
            Cube(c.x, c.y - 1, c.z),
            Cube(c.x, c.y, c.z + 1),
            Cube(c.x, c.y, c.z - 1),
        ]
    )


def count_neighbors(cubes: FrozenSet[Cube], c: Cube) -> int:
    return sum(1 for n in sides(c) if n in cubes)


print(f"part 1:\t{sum(6 - count_neighbors(cubes, c) for c in cubes)}")

seen: Set[Cube] = set()
q: List[Cube] = [Cube(-1, -1, -1)]

while q:
    cur = q.pop()
    q += [s for s in (sides(cur) - cubes - seen) if all(-1 <= c <= 25 for c in s)]
    seen |= {cur}

print(f"part 2:\t{sum((s in seen) for c in cubes for s in sides(c))}")
