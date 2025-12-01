from dataclasses import dataclass


@dataclass
class Coord:
	x: int
	y: int


def manhattan_distance(c1: Coord, c2: Coord) -> int:
	return abs(c1.x - c2.x) + abs(c1.y - c2.y)


p: list[Coord] = []

x = 0
y = 0

for c in input():
	if c == "P":
		p.append(Coord(x, y))

	if c == ",":
		x = 0
		y += 1
	else:
		x += 1

print(manhattan_distance(p[0], p[1]))
