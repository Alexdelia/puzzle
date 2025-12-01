#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

cycles = 0
x = 1
strength = 0

time = 0

s = "."

print("part 2:")
for l in lines:
	v = 0

	if l.startswith("noop"):
		time = 1
	else:
		v = int(l.split(" ")[-1])
		time = 2

	for i in range(time):
		cycles += 1
		if (
			cycles == 20
			or cycles == 60
			or cycles == 100
			or cycles == 140
			or cycles == 180
			or cycles == 220
		):
			strength += x * cycles
		if cycles % 40 == 0:
			print(s)
			s = ""
		if set([cycles % 40 - 1]).intersection(set([x - 1, x, x + 1])):
			s += "#"
		else:
			s += "."

	x += v

print(f"part 1:\t{strength}")
