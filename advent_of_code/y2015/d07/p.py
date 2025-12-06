#!/usr/bin/env python3

from __future__ import annotations

import re
from pathlib import Path

import numpy as np
from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", Path(__file__).parent.name))
YEAR = int(re.sub(r"[^0-9]", "", Path(__file__).parent.parent.name))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

circuit: dict[str, Wire] = {}


def to_value(x: str) -> np.uint16:
	if x.isdigit():
		return np.uint16(x)
	return circuit[x].eval()


class Wire:
	def __init__(self, name: str, cmd: str) -> None:
		self.name: str = name
		self.cmd: str = cmd
		self.value: np.uint16 | None = None

	def eval(self) -> np.uint16:
		if self.value is not None:
			return self.value

		if self.cmd.isdigit():
			self.value = np.uint16(self.cmd)
		elif "AND" in self.cmd:
			a, b = self.cmd.split(" AND ")
			self.value = to_value(a) & to_value(b)
		elif "OR" in self.cmd:
			a, b = self.cmd.split(" OR ")
			self.value = to_value(a) | to_value(b)
		elif "LSHIFT" in self.cmd:
			a, b = self.cmd.split(" LSHIFT ")
			self.value = to_value(a) << to_value(b)
		elif "RSHIFT" in self.cmd:
			a, b = self.cmd.split(" RSHIFT ")
			self.value = to_value(a) >> to_value(b)
		elif "NOT" in self.cmd:
			a = self.cmd.split("NOT ")[1]
			self.value = ~to_value(a)
		else:
			self.value = to_value(self.cmd)

		return self.value


for l in lines:
	cmd, name = l.split(" -> ")
	circuit[name] = Wire(name, cmd)

a = circuit["a"].eval()
print(f"part 1:\t{a}")

for v in circuit.values():
	v.value = None
circuit["b"].value = a

print(f"part 2:\t{circuit['a'].eval()}")
