#!/usr/bin/env python3

from __future__ import annotations

import re
from os.path import dirname
from typing import List, Tuple

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

ms: List[Monkey] = []

mtrick = 1

class Throw:
    def __init__(self, test: Tuple[str, str, str]):
        self.divisor = int(test[0].split()[-1])
        self.t_id = int(test[1].split()[-1])
        self.f_id = int(test[2].split()[-1])

        global mtrick
        mtrick *= self.divisor
    
    def test(self, item: int) -> int:
        if item % self.divisor == 0:
            return self.t_id
        else:
            return self.f_id


class Operate:
    def __init__(self, operation: str):
        s = re.sub("  Operation: new = ", '', operation).split()
        
        assert s[0] == "old"
        if s[2] == "old":
            assert s[1] == "*"

        try:
            self.rhs = int(s[2])
        except ValueError:
            self.rhs = s[2]

        if s[1] == '+':
            self.operate = self.operate_add
        elif s[1] == '*':
            self.operate = self.operate_mul
    
    def operate_add(self, old: int) -> int:
        return old + self.rhs
    
    def operate_mul(self, old: int) -> int:
        if isinstance(self.rhs, int):
            return old * self.rhs
        else:
            return old * old

    # def operate(self, old: int) -> int:
    #     return eval(re.sub("old", str(old), self.operation))


class Monkey:
    def __init__(self, items: str, operation: str, test: Tuple[str, str, str]):
        self.items: List[int] = [int(x) for x in re.sub(',', '', re.sub("  Starting items: ", '', items)).split()]
        self.operation: Operate = Operate(operation)
        self.test: Throw = Throw(test)
        self.n_inspect: int = 0

    def run(self, relief: bool = False):
        while self.items:
            item = self.items.pop(0)
            self.n_inspect += 1
            item = self.operation.operate(item)
            if relief:
                item //= 3
            else:
                global mtrick
                item %= mtrick
            ms[self.test.test(item)].items.append(item)


i = 0
while i < len(lines):
    if "Starting items:" in lines[i]:
        ms.append(Monkey(lines[i], lines[i + 1], (lines[i + 2], lines[i + 3], lines[i + 4])))
        i += 4
    i += 1

print(mtrick)

for i in range(20):
    for m in ms:
        m.run(True)


inspected: List[int] = sorted([m.n_inspect for m in ms])
print(inspected)
print(f"part 1:\t{inspected[-1] * inspected[-2]}")

ms.clear()

i = 0
while i < len(lines):
    if "Starting items:" in lines[i]:
        ms.append(Monkey(lines[i], lines[i + 1], (lines[i + 2], lines[i + 3], lines[i + 4])))
        i += 4
    i += 1

for i in range(10000):
    print(i, end='\r')
    for m in ms:
        m.run(False)

inspected: List[int] = sorted([m.n_inspect for m in ms])
print(inspected)
print(f"part 2:\t{inspected[-1] * inspected[-2]}")
