#!/usr/bin/env python3

import re
from ast import literal_eval
from functools import cmp_to_key
from os.path import dirname
from typing import Any, List, Union

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

def comp(lhs: Union[List[int], int], rhs: Union[List[int], int]) -> int:
    if isinstance(lhs, int) and isinstance(rhs, int):
        if lhs < rhs:
            return -1
        elif lhs > rhs:
            return 1
        return 0

    if not isinstance(lhs, list):
        lhs = [lhs]
    if not isinstance(rhs, list):
        rhs = [rhs]

    i = 0
    while i < len(lhs) and i < len(rhs):
        c = comp(lhs[i], rhs[i])
        if c != 0:
            return c
        i += 1

    if len(lhs) < len(rhs):
        return -1
    elif len(lhs) > len(rhs):
        return 1
    return 0

t = 0
pair = 1
i = 0

while i < len(lines):
    lhs = literal_eval(lines[i])
    rhs = literal_eval(lines[i + 1])

    if comp(lhs, rhs) == -1:
        t += pair

    i += 3
    pair += 1

print(f"part 1:\t{t}")

packets: List[List[Any]] = [
    [[2]],
    [[6]]
]

for l in lines:
    if len(l) == 0:
        continue
    else:
        packets.append(literal_eval(l))

packets.sort(key=cmp_to_key(comp))

d1 = -1
d2 = -1

for i, p in enumerate(packets):
    if p == [[2]]:
        d1 = i + 1
    elif p == [[6]]:
        d2 = i + 1

print(f"part 2:\t{d1 * d2}")
