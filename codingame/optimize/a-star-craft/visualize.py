#!/usr/bin/env python3
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent

MAP_WIDTH = 19
MAP_HEIGHT = 10

VALIDATOR_DIR = SCRIPT_DIR / "validator"
OUTPUT_DIR = SCRIPT_DIR / "output"
SOLUTION_FILE = "solution.txt"

VOID_CHAR = "#"
EMPTY_CHAR = "."
ROBOT_CHAR = {"U", "R", "D", "L"}
ARROW_CHAR = {"u": "U", "r": "R", "d": "D", "l": "L"}

GLYPH = {
	"U": "^",
	"R": ">",
	"D": "v",
	"L": "<",
}

RESET = "\033[0m"
VOID_STYLE = "\033[100m"
EMPTY_STYLE = "\033[0m"
ROBOT_STYLE = "\033[30;47m"
ARROW_STYLE = "\033[1;32m"

CELL_WIDTH = 2


def cell_text(glyph: str) -> str:
	return glyph.ljust(CELL_WIDTH) if glyph else " " * CELL_WIDTH


def render_void() -> str:
	return VOID_STYLE + cell_text("") + RESET


def render_empty() -> str:
	return EMPTY_STYLE + cell_text("") + RESET


def render_robot(heading: str) -> str:
	return ROBOT_STYLE + cell_text(GLYPH[heading]) + RESET


def render_arrow(direction: str) -> str:
	return ARROW_STYLE + cell_text(GLYPH[direction]) + RESET


def read_map(validator: str) -> list[str]:
	path = VALIDATOR_DIR / f"{validator}.txt"
	return path.read_text().splitlines()[:MAP_HEIGHT]


def read_arrow(validator: str) -> dict[tuple[int, int], str]:
	path = OUTPUT_DIR / validator / SOLUTION_FILE
	arrow: dict[tuple[int, int], str] = {}
	try:
		token = path.read_text().split()
	except FileNotFoundError:
		return arrow
	for i in range(0, len(token) - 2, 3):
		x = int(token[i])
		y = int(token[i + 1])
		arrow[(x, y)] = token[i + 2]
	return arrow


def render(validator: str) -> None:
	grid = read_map(validator)
	arrow = read_arrow(validator)
	for y in range(MAP_HEIGHT):
		row = grid[y] if y < len(grid) else ""
		line = ""
		for x in range(MAP_WIDTH):
			c = row[x] if x < len(row) else VOID_CHAR
			if (x, y) in arrow:
				line += render_arrow(arrow[(x, y)])
			elif c == VOID_CHAR:
				line += render_void()
			elif c in ROBOT_CHAR:
				line += render_robot(c)
			elif c in ARROW_CHAR:
				line += render_arrow(ARROW_CHAR[c])
			else:
				line += render_empty()
		print(line)


def main() -> None:
	if len(sys.argv) != 2:
		print(f"usage: {sys.argv[0]} <validator>", file=sys.stderr)
		sys.exit(1)
	render(sys.argv[1])


if __name__ == "__main__":
	main()
