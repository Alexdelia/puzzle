#!/usr/bin/env python3

from aocd import get_data

DAY = 2
YEAR = 2022
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

score = 0

dc = {
    0: "A",
    1: "B",
    2: "C",
}

dn = {
    "A": 0,
    "B": 1,
    "C": 2,
}

for l in lines:
    op, me = l.split(" ")
    # rock paper scissors:
    # op: A, B, C
    # me: X, Y, Z
    op = dn[op]
    # print(op, me)

    if me == "Y":   # draw
        score += 3
        me = dc[op]
    elif me == "Z": # win
        score += 6
        me = dc[(op + 1) % 3]
    elif me == "X": # lose
        me = dc[(op - 1) % 3]

    assert isinstance(me, str)
    
    if me == 'A':
        score += 1
    elif me == 'B':
        score += 2
    elif me == 'C':
        score += 3

print(score)
