#!/usr/bin/env python3

from aocd import get_data

DAY = 2
YEAR = 2022
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

score = 0

for l in lines:
	op, me = l.split(" ")
	# rock paper scissors:
	# op: A, B, C
	# me: X, Y, Z
	me = me.replace("X", "A").replace("Y", "B").replace("Z", "C")
	# print(op, me)

	if me == op:
		score += 3
	elif me == "C" and op == "B" or me == "B" and op == "A" or me == "A" and op == "C":
		score += 6

	if me == "A":
		score += 1
	elif me == "B":
		score += 2
	elif me == "C":
		score += 3

print(score)
