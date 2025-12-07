#!/usr/bin/env python3

import sys
from enum import Enum

SECOND_DIVIDER = 100

TEST_SIMULATION_SECOND = 1000
PART_1_SIMULATION_SECOND = 2503


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
	) -> None:
		self.name = name
		self.speed = speed
		self.fly_time = fly_time
		self.rest_time = rest_time
		self.state = state
		self.flying_for = flying_for
		self.resting_for = resting_for
		self.traveled = traveled

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


def simulate(deer_list: list[Deer], simulation_second: int) -> None:
	for _ in range(simulation_second):
		for deer in deer_list:
			deer.simulate_second()


def solve(
	data: str,
	simulation_second: int,
	visualize: bool = False,
) -> tuple[int, int]:
	deer_list = parse(data)

	simulate(deer_list, simulation_second)

	p1 = max(deer.traveled for deer in deer_list)

	return (p1, 0)


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


expected = (1120, 0)
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
