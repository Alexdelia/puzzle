#!/usr/bin/env python3

import re
import sys
import json


def parse_obj(obj: dict[str, any] | list[any] | int | str) -> int:
	if isinstance(obj, int):
		return obj

	if isinstance(obj, list):
		return sum(parse_obj(item) for item in obj)

	if isinstance(obj, dict):
		if "red" in obj.values():
			return 0

		return sum(parse_obj(value) for value in obj.values())

	return 0


def solve(data: str) -> tuple[int, int]:
	number_regex = re.compile(r"\-?\d+")

	p1 = 0

	for match in number_regex.findall(data):
		p1 += int(match)

	# === part 2 ===

	data = json.loads(data)

	p2 = parse_obj(data)

	return (p1, p2)


sys.path.append("../..")
from get_data import get_data

p1, p2 = solve(get_data())
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
