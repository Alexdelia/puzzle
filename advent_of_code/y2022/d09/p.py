#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

moves: list[str] = []

for l in lines:
	dir, n = l.split()
	for i in range(int(n)):
		moves.append(dir)

print(f"len(moves): {len(moves)}")


# check if t is at the same position as h or in the 8 surrounding squares
def check_9(head: tuple[int, int], tail: tuple[int, int]) -> bool:
	x, y = tail
	for dx in range(-1, 2):
		for dy in range(-1, 2):
			if head == (x + dx, y + dy):
				return True
	return False


def check_4(head: tuple[int, int], tail: tuple[int, int]) -> bool:
	x, y = tail
	for dx, dy in [(1, 0), (0, 1), (-1, 0), (0, -1)]:
		if head == (x + dx, y + dy):
			return True
	return False


# the tail is too far away from the head, move it in the 8 surrounding squares of the head
def find_4(head: tuple[int, int], tail: tuple[int, int]) -> tuple[int, int] | None:
	x, y = tail
	for dx in range(-1, 2):
		for dy in range(-1, 2):
			if check_4(head, (x + dx, y + dy)):
				return (x + dx, y + dy)
	return None


def find_9(head: tuple[int, int], tail: tuple[int, int]) -> tuple[int, int] | None:
	x, y = tail
	for dx in range(-1, 2):
		for dy in range(-1, 2):
			if check_9(head, (x + dx, y + dy)):
				return (x + dx, y + dy)
	return None


hx, hy = 0, 0
tx, ty = 0, 0
visited: set[tuple[int, int]] = set([(0, 0)])

for m in moves:
	if m == "U":
		hy += 1
	elif m == "D":
		hy -= 1
	elif m == "R":
		hx += 1
	elif m == "L":
		hx -= 1

	if check_9((hx, hy), (tx, ty)):
		continue

	new = find_4((hx, hy), (tx, ty))
	if not new:
		print("panic")
		exit(1)

	tx, ty = new
	visited.add((tx, ty))

print(f"part 1:\t{len(visited)}")

hx, hy = 0, 0
tails: list[tuple[int, int]] = [(0, 0)] * 9
visited: set[tuple[int, int]] = set([(0, 0)])

for m in moves:
	if m == "U":
		hy += 1
	elif m == "D":
		hy -= 1
	elif m == "R":
		hx += 1
	elif m == "L":
		hx -= 1

	for i in range(len(tails)):
		if i == 0:
			head = (hx, hy)
		else:
			head = tails[i - 1]

		if check_9(head, tails[i]):
			continue

		new = find_4(head, tails[i])
		if not new:
			new = find_9(head, tails[i])
			if not new:
				print("panic")
				exit(1)

		tails[i] = new

	visited.add(tails[-1])

print(f"part 2:\t{len(visited)}")
