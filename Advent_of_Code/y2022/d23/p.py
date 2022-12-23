#!/usr/bin/env python3

import re
from os.path import dirname
from typing import List, Tuple

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

DATA_EXAMPLE = """\
..............
..............
.......#......
.....###.#....
...#...#.#....
....#...##....
...#.###......
...##.#.##....
....#..#......
..............
..............
..............\
"""

SIZE = 1000
GRID = List[List[str]]


def parse(data: str) -> GRID:
    grid = [['.' for _ in range(SIZE)] for _ in range(SIZE)]

    x = (SIZE - len(data.splitlines())) // 2
    for l in data.splitlines():
        y = (SIZE - len(data.splitlines()[0])) // 2
        for c in l:
            grid[x][y] = c
            y += 1
        x += 1

    return grid


def decide(grid: GRID, x: int, y: int) -> Tuple[int, int]:
    if grid[x - 1][y - 1] == '.' and grid[x - 1][y] == '.' and grid[x - 1][y + 1] == '.':
        return (x - 1, y)
    elif grid[x + 1][y - 1] == '.' and grid[x + 1][y] == '.' and grid[x + 1][y + 1] == '.':
        return (x + 1, y)
    elif grid[x - 1][y - 1] == '.' and grid[x][y - 1] == '.' and grid[x + 1][y - 1] == '.':
        return (x, y - 1)
    elif grid[x - 1][y + 1] == '.' and grid[x][y + 1] == '.' and grid[x + 1][y + 1] == '.':
        return (x, y + 1)
    else:
        return (x, y)


def simulate(g: GRID, round: int) -> GRID:
    for _ in range(round):
        gp: GRID = [['.' for _ in range(SIZE)] for _ in range(SIZE)]

        for x in range(SIZE):
            for y in range(SIZE):
                if g[x][y] == '#':
                    nx, ny = decide(g, x, y)
                    gp[nx][ny] = '#'
        
        g = gp
    
    return g


def calc(g: GRID) -> int:
    # find how many '.' in the smallest rectangle containing all '#'
    x1, x2, y1, y2 = SIZE, 0, SIZE, 0
    for x in range(SIZE):
        for y in range(SIZE):
            if g[x][y] == '#':
                x1, x2 = min(x1, x), max(x2, x)
                y1, y2 = min(y1, y), max(y2, y)

    for x in range(x1 - 3, x2 + 1 + 3):
        print(''.join(g[x][y1 - 3:y2 + 1 + 3]))

    t = 0    
    for x in range(x1, x2 + 1):
        for y in range(y1, y2 + 1):
            if g[x][y] == '.':
                t += 1
    return t


g = parse(DATA_EXAMPLE)
calc(g)
g = simulate(g, 10)
calc(g)
