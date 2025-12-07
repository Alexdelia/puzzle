#!/usr/bin/env python3

import os
import sys
from enum import Enum
from time import sleep

TEST_SIMULATION_SECOND = 1000
PART_1_SIMULATION_SECOND = 2503

FLY_DELAY = 0.05

COLOR_SELECTION = [
	"\033[38;2;255;64;64m",
	"\033[38;2;64;255;64m",
	"\033[38;2;64;64;255m",
	"\033[38;2;255;255;64m",
	"\033[38;2;64;255;255m",
	"\033[38;2;255;64;255m",
	"\033[38;2;255;128;64m",
	"\033[38;2;64;255;128m",
	"\033[38;2;128;64;255m",
]

DEER_SYMBOL = " "
LEADING_SYMBOL = "󰔸 "

REST_SYMBOL = "󰒲 "

FLY_COLOR = "\033[1m"
REST_COLOR = "\033[2m"
LEADING_COLOR = "\033[0;1;38;2;255;215;0m"

WALL_COLOR = "\033[0;2m"


class State(Enum):
	FLYING = 1
	RESTING = 2


class Deer:
	name: str
	speed: int
	fly_time: int
	rest_time: int

	state: State
	flying_for: int
	resting_for: int
	traveled: int

	score: int

	leading: bool

	def __init__(  # noqa: PLR0913
		self,
		name: str,
		speed: int,
		fly_time: int,
		rest_time: int,
		state: State = State.FLYING,
		flying_for: int = 0,
		resting_for: int = 0,
		traveled: int = 0,
		score: int = 0,
		leading: bool = False,
	) -> None:
		self.name = name
		self.speed = speed
		self.fly_time = fly_time
		self.rest_time = rest_time
		self.state = state
		self.flying_for = flying_for
		self.resting_for = resting_for
		self.traveled = traveled
		self.score = score
		self.leading = leading

	def __eq__(self, other: object) -> bool:
		if not isinstance(other, Deer):
			return NotImplemented

		return any(key != getattr(other, key) for key in self.__dict__)

	def __hash__(self) -> int:
		return hash(self.__dict__)

	def simulate_second(self) -> None:
		if self.state == State.FLYING:
			self.fly()
		else:
			self.rest()

	def fly(self) -> None:
		self.traveled += self.speed
		self.flying_for += 1

		if self.flying_for >= self.fly_time:
			self.state = State.RESTING
			self.flying_for = 0
			self.resting_for = 0

	def rest(self) -> None:
		self.resting_for += 1

		if self.resting_for >= self.rest_time:
			self.state = State.FLYING
			self.flying_for = 0
			self.resting_for = 0


def parse(data: str) -> list[Deer]:
	lines = data.splitlines()
	rule: list[Deer] = []

	for line in lines:
		p = line.split()
		rule.append(
			Deer(
				name=p[0],
				speed=int(p[3]),
				fly_time=int(p[6]),
				rest_time=int(p[13]),
			)
		)

	return rule


def visualize_second(  # noqa: PLR0915
	second: int,
	max_second: int,
	deer_list: list[Deer],
	max_distance: int,
) -> None:
	w = os.get_terminal_size().columns

	buf = ""

	if second == 0:
		buf += "\033c"
	else:
		h = 7 + 6 + len(deer_list) * 5 + 2
		buf += f"\033[{h}F"

	buf += WALL_COLOR + "╭" + "─" * (w - 2) + "╮\n"

	# second counter
	percent = second / max_second * 100
	counter = f"\033[0;1m{second}\033[0m/{max_second} \033[1m{percent:.2f}\033[0m%"
	counter_len = len(f"{second}/{max_second} {percent:.2f}%")
	left_pad = (w - counter_len) // 2
	right_pad = w - counter_len - left_pad
	buf += (
		WALL_COLOR
		+ "│"
		+ " " * (left_pad - 1)
		+ counter
		+ " " * (right_pad - 1)
		+ WALL_COLOR
		+ "│\n"
	)

	def get_deer_color(deer: Deer, index: int) -> str:
		color: str = COLOR_SELECTION[index % len(COLOR_SELECTION)]
		if deer.state == State.FLYING:
			color += FLY_COLOR
		else:
			color += REST_COLOR
		return color

	# deer scoreboard
	score_board_w = (w - 2) // len(deer_list)
	g_pad = (w - 2 - score_board_w * len(deer_list)) // 2
	l_pad = g_pad + 1
	r_pad = g_pad
	buf += (
		WALL_COLOR
		+ "│"
		+ " " * l_pad
		+ (("╭" + "─" * (score_board_w - 3) + "╮ ") * len(deer_list))
		+ " " * r_pad
		+ "│\n"
	)

	## deer name and state
	buf += WALL_COLOR + "│" + " " * l_pad
	for i, deer in enumerate(deer_list):
		color = get_deer_color(deer, i)

		symbol = DEER_SYMBOL if deer.state == State.FLYING else REST_SYMBOL

		buf += (
			WALL_COLOR
			+ "│   "
			+ color
			+ f"{deer.name:^{score_board_w - 8}}"
			+ symbol
			+ WALL_COLOR
			+ "│ "
		)
	buf += " " * r_pad + "│\n"

	## deer distance
	buf += WALL_COLOR + "│" + " " * l_pad
	for deer in deer_list:
		leading = LEADING_COLOR + LEADING_SYMBOL if deer.leading else "  "
		buf += (
			WALL_COLOR
			+ f"│{leading}\033[0;2mkm: \033[0;1m{deer.traveled:>{score_board_w - 11}}  "
			+ WALL_COLOR
			+ "│ "
		)
	buf += " " * r_pad + "│\n"

	## deer score
	buf += WALL_COLOR + "│" + " " * l_pad
	top_score = max(deer.score for deer in deer_list)
	for deer in deer_list:
		leading_dist = LEADING_COLOR + "+1" if deer.leading else "  "
		leading_score = (
			LEADING_COLOR + LEADING_SYMBOL if deer.score == top_score else "  "
		)

		buf += (
			WALL_COLOR
			+ "│"
			+ str(leading_score)
			+ f"\033[0;2mpoints: \033[0;1m{deer.score:>{score_board_w - 15}}"
			+ str(leading_dist)
			+ WALL_COLOR
			+ "│ "
		)
	buf += " " * r_pad + "│\n"

	# close scoreboard
	buf += (
		WALL_COLOR
		+ "│"
		+ " " * l_pad
		+ (("╰" + "─" * (score_board_w - 3) + "╯ ") * len(deer_list))
		+ " " * r_pad
		+ "│\n"
	)

	# spacing
	buf += WALL_COLOR + "│" + " " * (w - 2) + "│\n"

	len_deer_symbol = len(DEER_SYMBOL)
	# distance visualization
	for i, deer in enumerate(deer_list):
		ratio = deer.traveled / max_distance
		pos = int(ratio * (w - 2))

		l_pad = min(pos, (w - 2) - len_deer_symbol)
		r_pad = (w - 2) - pos - len_deer_symbol

		color = get_deer_color(deer, i)
		symbol = DEER_SYMBOL if deer.state == State.FLYING else REST_SYMBOL

		buf += (
			WALL_COLOR
			+ "│"
			+ "." * l_pad
			+ color
			+ symbol
			+ " " * r_pad
			+ WALL_COLOR
			+ "│\n"
		)

	buf += WALL_COLOR + "╰" + "─" * (w - 2) + "╯\n"

	print(buf, end="\033[0m")

	if any(deer.state == State.FLYING for deer in deer_list):
		sleep(FLY_DELAY)


def simulate(
	deer_list: list[Deer],
	simulation_second: int,
	visualize: bool = False,
	max_distance: int = 0,
) -> None:
	if visualize:
		visualize_second(0, simulation_second, deer_list, max_distance)

	for second in range(simulation_second):
		lead_dist = 0

		for deer in deer_list:
			deer.simulate_second()

			lead_dist = max(lead_dist, deer.traveled)

		for deer in deer_list:
			if deer.traveled == lead_dist:
				deer.score += 1
				deer.leading = True
			else:
				deer.leading = False

		if visualize:
			visualize_second(second + 1, simulation_second, deer_list, max_distance)


def solve(
	data: str,
	simulation_second: int,
	visualize: bool = False,
) -> tuple[int, int]:
	deer_list = parse(data)

	max_distance = 0
	if visualize:
		max_distance, _ = solve(data, simulation_second, visualize=False)

	simulate(deer_list, simulation_second, visualize, max_distance)

	p1 = max(deer.traveled for deer in deer_list)
	p2 = max(deer.score for deer in deer_list)

	return (p1, p2)


test = """\
Comet can fly 14 km/s for 10 seconds, but then must rest for 127 seconds.
Dancer can fly 16 km/s for 11 seconds, but then must rest for 162 seconds.
"""

expected_parse: list[Deer] = [
	Deer(name="Comet", speed=14, fly_time=10, rest_time=127),
	Deer(name="Dancer", speed=16, fly_time=11, rest_time=162),
]
got_parse = parse(test)
assert expected_parse == got_parse, (
	f"parse test failed:\nexpected: {expected_parse}\ngot: {got_parse}"
)

expected_simulation_traveled = [
	1120,
	1056,
]
deer_list = got_parse
simulate(deer_list, TEST_SIMULATION_SECOND)
for i, deer in enumerate(deer_list):
	assert expected_simulation_traveled[i] == deer.traveled, (
		f"simulation test failed for deer '{deer.name}': "
		f"expected {expected_simulation_traveled[i]}, got {deer.traveled}"
	)


expected = (1120, 689)
got = solve(test, TEST_SIMULATION_SECOND)
assert expected[0] == got[0], (
	f"part 1 test failed: expected {expected[0]}, got {got[0]}"
)
assert expected[1] == got[1], (
	f"part 2 test failed: expected {expected[1]}, got {got[1]}"
)


sys.path.append("../..")
from get_data import get_data

p1, p2 = solve(get_data(), PART_1_SIMULATION_SECOND, visualize=True)
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
