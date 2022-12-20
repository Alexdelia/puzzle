#!/usr/bin/env python3

import re
from copy import deepcopy
from os.path import dirname
from typing import List, Tuple

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

DATA_EXAMPLE = "1\n2\n-3\n3\n-2\n0\n4"

class Num:
    def __init__(self, n: int, t: int, i: int):
        self.n = n
        self.t = t
        self.i = i
    
    def update(self, n: int, t: int, i: int):
        self.n = n
        self.t = t
        self.i = i


def parse(data: str) -> List[Num]:
    lines = data.splitlines()
    sequence: List[Num] = []

    for i, l in enumerate(lines):
        n = int(l)
        t = 0
        for num in sequence:
            if num.n == n:
                t += 1
        sequence.append(Num(n, t, i))
    
    return sequence


def find_num(sequence: List[Num], mix: List[Num], num: Num) -> int:
    if mix[num.i].n == num.n and mix[num.i].t == num.t:
        return num.i

    bi = i = num.i

    while bi >= 0 or i < len(sequence):
        if bi >= 0:
            if mix[bi].n == num.n and mix[bi].t == num.t:
                return bi
            bi -= 1
        if i < len(sequence):
            if mix[i].n == num.n and mix[i].t == num.t:
                return i
            i += 1
    raise Exception("Not found")


def solve(sequence: List[Num], mix: List[Num], iteration: int) -> Tuple[int, int, int]:
    for it in range(iteration):
        for iv, num in enumerate(sequence):
            print(f"{it + 1} {iv + 1} / {iteration} {len(sequence)}", end='\r')

            i = find_num(sequence, mix, num)
            n = deepcopy(mix[i])
    
            if n.n > 0:
                for x in range(i, i + n.n):
                    tmp = mix[(x + 1) % len(mix)]
                    mix[x % len(mix)].update(tmp.n, tmp.t, x)
                idx = (i + n.n) % len(mix)
                mix[idx].update(n.n, n.t, idx)
            elif n.n < 0:
                for x in range(i, i + n.n, -1):
                    tmp = mix[(x - 1) % len(mix)]
                    mix[x % len(mix)].update(tmp.n, tmp.t, x)
                idx = (i + n.n) % len(mix)
                mix[idx].update(n.n, n.t, idx)

    i = find_num(sequence, mix, Num(0, 0, 0))
    return (mix[(i + 1000) % len(mix)].n,
            mix[(i + 2000) % len(mix)].n,
            mix[(i + 3000) % len(mix)].n)


sequence = parse(DATA_EXAMPLE)
n = solve(sequence, deepcopy(sequence), 1)
assert n == (4, -3, 2), f"Expected (4, -3, 2), got {n}"
assert sum(n) == 3, f"Expected 3, got {sum(n)}"

sequence = parse(DATA)
n = solve(sequence, deepcopy(sequence), 1)
print(f"\npart 1:\t{sum(n)}")

KEY = 811589153
sequence = parse(DATA_EXAMPLE)
for num in sequence:
    num.n *= KEY
n = solve(sequence, deepcopy(sequence), 10)
assert n == (811589153, 2434767459, 1623178306), f"Expected (811589153, 2434767459, 1623178306), got {n}"
assert sum(n) == 1623178306, f"Expected 1623178306, got {sum(n)}"

sequence = parse(DATA)
for num in sequence:
    num.n *= KEY
n = solve(sequence, deepcopy(sequence), 10)
print(f"\npart 2:\t{sum(n)}")
