#!/usr/bin/env python3

import re
from collections import namedtuple
from copy import deepcopy
from os.path import dirname
from typing import List, Tuple

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

Map = List[List[str]]
Actions = List[Tuple[int, str]]

DATA_EXAMPLE = """        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

10R5L5R10L4R5L5"""

Coord = namedtuple('Coord', 'x y')
dir: Tuple[Coord, Coord, Coord, Coord] = (Coord(0, 1), Coord(1, 0), Coord(0, -1), Coord(-1, 0))


def parse(data: str) -> Tuple[Map, Actions]:
    lines = data.splitlines()
    m = [list(l) for l in lines[:-2]]
    a = re.findall(r'([0-9]+)([RL])', lines[-1])
    a = [(int(n), t) for n, t in a]
    return m, a

def solve(m: Map, a: Actions, draw: bool = False) -> Tuple[int, int, int]:
    if draw:
        md = deepcopy(m)
        for l in md:
            print(''.join(l))
        print()
        p = ('>', 'v', '<', '^')

    for i, c in enumerate(m[0]):
        if c == '.':
            break
    else:
        raise ValueError("no start found")
    
    x, y = 0, i
    d = 0
    for n, turn in a:
        for _ in range(n):
            if draw:
                md[x][y] = p[d]

            nx, ny = x + dir[d].x, y + dir[d].y
            if nx < 0 or ny < 0 or nx >= len(m) or ny >= len(m[nx]) or m[nx][ny] == ' ':
                # wrap around
                nx, ny = nx - dir[d].x, ny - dir[d].y
                while nx >= 0 and ny >= 0  and nx < len(m) and ny < len(m[nx]) and m[nx][ny] != ' ':
                    nx, ny = nx - dir[d].x, ny - dir[d].y
                nx, ny = nx + dir[d].x, ny + dir[d].y
                if m[nx][ny] != '.' and m[nx][ny] != '#':
                    raise ValueError(f"wrapped unknown char '{m[nx][ny]}'")
                elif m[nx][ny] == '.':
                    x, y = nx, ny

            elif m[nx][ny] == '.':
                x, y = nx, ny
            elif m[nx][ny] == '#':
                break
            else:
                raise ValueError(f"unknown char '{m[nx][ny]}'")
        
        if turn == 'R':
            d = (d + 1) % 4
        elif turn == 'L':
            d = (d - 1) % 4
        else:
            raise ValueError(f"unknown turn {turn}")

    if draw:
        for l in md:
            print(''.join(l))
        print()

    return x + 1, y + 1, d


m, a = parse(DATA_EXAMPLE)
x, y, d = solve(m, a, True)
assert (x, y, d) == (6, 8, 0), f"expected (6, 8, 0), got {(x, y, d)}"

m, a = parse(DATA)
x, y, d = solve(m, a)
print(f"x == {x}\ny == {y}\ndir == {d}")
print(f"part 1:\t{x * 1000 + y * 4 + d}")
