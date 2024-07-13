#!/usr/bin/env python3

import re
from os.path import dirname
from typing import FrozenSet, List, Tuple, Union
from time import sleep, time

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

DATA_EXAMPLE = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>"

START = time()

Coord = Tuple[int, int]

SIZE_W = 7
SIZE_H = 10_000

def print_grid(m: List[List[bool]], h: int):
    s = ""
    for y in range(h + 1, 0, -1):
        s += "|"
        for x in range(SIZE_W):
            s += "🟩" if m[x][y] else "  "
        s += "|\n"
    s += "+" + "-" * (SIZE_W * 2) + "+\n"
    print(s)


class Rock:
    def __init__(self, rock: FrozenSet[Coord]):
        self.rock = rock
        self.w = max(x for x, _ in rock) + 1
        self.h = max(y for _, y in rock) + 1
    
    def __repr__(self):
        lines: List[List[str]] = [["🟤" for _ in range(self.w)] for _ in range(self.h)]
        for x, y in self.rock:
            lines[y][x] = "🟩"
        
        s = f"\033[1mw\033[35m{self.w}\033[0m\t\033[1mh\033[35m{self.h}\033[0m\n"
        for i in range(len(lines) - 1, -1, -1):
            s += "".join(lines[i]) + '\n'

        return s
    
    def can_fall(self, grid: List[List[bool]], x: int, y: int) -> bool:
        for rx, ry in self.rock:
            if grid[x + rx][y + ry - 1]:
                return False
        return True

    def can_move(self, grid: List[List[bool]], x: int, y: int, dir: str) -> Union[Coord, None]:
        assert dir in "<>" and len(dir) == 1

        if dir == "<" and x > 0:
            for rx, ry in self.rock:
                if grid[x + rx - 1][y + ry]:
                    return None
            return (x - 1, y)
        elif dir == ">" and x < SIZE_W - self.w:
            for rx, ry in self.rock:
                if grid[x + rx + 1][y + ry]:
                    return None
            return (x + 1, y)
        return None
    
    def move(self, grid: List[List[bool]], c: Coord, actions: str, mi: int) -> Tuple[Coord, int]:
        x, y = c
        res = self.can_move(grid, x, y, actions[mi])
        if res is not None:
            x, y = res
        mi = (mi + 1) % len(actions)

        return ((x, y), mi)

    def place(self, m: List[List[bool]], x: int, y: int):
        for rx, ry in self.rock:
            m[x + rx][y + ry] = True


rocks: List[Rock] = [
    Rock(
        frozenset([
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
        ]),
    ),
    Rock(
        frozenset([
            (0, 1),
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 1),
        ]),
    ),
    Rock(
        frozenset([
            (0, 0),
            (1, 0),
            (2, 0),
            (2, 1),
            (2, 2),
        ]),
    ),
    Rock(
        frozenset([
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
        ]),
    ),
    Rock(
        frozenset([
            (0, 0),
            (0, 1),
            (1, 0),
            (1, 1),
        ]),
    ),
]


# for i, r in enumerate(rocks):
#     print(f"Rock {i}:\n{r}")


def debug(grid: List[List[bool]], h: int, ri: int, c: Coord):
    copy = [[False for _ in range(SIZE_H)] for _ in range(SIZE_W)]
    for x in range(SIZE_W):
        for y in range(SIZE_H):
            copy[x][y] = grid[x][y]
    rocks[ri].place(copy, *c)
    print_grid(copy, h)


def solve(actions: str, turns: int) -> int:

    grid: List[List[bool]] = [[False for _ in range(SIZE_H)] for _ in range(SIZE_W)]

    for x in range(SIZE_W):
        grid[x][0] = True

    total = 0
    h = 4
    ri = 0
    mi = 0

    for _ in range(turns):
        # print(f"Rock {ri}:\n{rocks[ri]}")
        # print(f"H: \033[32;1m{h}\033[0m")

        x = 2
        y = h

        # debug(grid, h, ri, (x, y))

        (x, y), mi = rocks[ri].move(grid, (x, y), actions, mi)

        # debug(grid, h, ri, (x, y))

        while rocks[ri].can_fall(grid, x, y):
            y -= 1
    
            # debug(grid, h, ri, (x, y))

            (x, y), mi = rocks[ri].move(grid, (x, y), actions, mi)
    
            # debug(grid, h, ri, (x, y))

        h = max(y + rocks[ri].h + 3, h)

        # print_grid(grid, h)
        # sleep(0.5)

        for fy in range(y, y + rocks[ri].h):
            if all(grid[x][fy] for x in range(SIZE_W)):
                total += fy
                print(f"\ntotal:\t{total}")
                # clear all under and move all rocks above down to full bottom
                for ry in range(fy, h):
                    for rx in range(SIZE_W):
                        grid[rx][ry - fy] = grid[rx][ry]
                        grid[rx][ry] = False
                h -= fy

        rocks[ri].place(grid, x, y)
        ri = (ri + 1) % len(rocks)

    return total + h - 4

assert solve(DATA_EXAMPLE, 2022) == 3068, f"{solve(DATA_EXAMPLE, 2022)} != 3068"
print(f"part 1:\t{solve(DATA, 2022)}")
assert solve(DATA_EXAMPLE, 1000000000000) == 1514285714288
