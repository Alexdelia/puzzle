#!/usr/bin/env python3

import sys
from itertools import permutations

Rule = dict[str, dict[str, int]]


def build_rule(data: str) -> Rule:
	rules: Rule = {}

	for line in data.splitlines():
		from_name, _, sign, n, *_, to_name = line[:-1].split()

		if from_name not in rules:
			rules[from_name] = {}

		rules[from_name][to_name] = int(n) * (-1 if sign == "lose" else 1)

	return rules


def calc_happiness(rule: Rule, persons: list[str]) -> int:
	h = 0

	for i in range(len(persons)):
		left = persons[i - 1]
		right = persons[(i + 1) % len(persons)]
		h += rule[persons[i]][left] + rule[persons[i]][right]

	return h


def max_happiness(rule: Rule) -> int:
	persons = list(rule.keys())
	return max(calc_happiness(rule, perm) for perm in permutations(persons))


def solve(data: str) -> tuple[int, int]:
	rule = build_rule(data)

	p1 = max_happiness(rule)

	# === part 2 ===

	persons = list(rule.keys())
	rule["me"] = {}
	for p in persons:
		rule[p]["me"] = 0
		rule["me"][p] = 0

	p2 = max_happiness(rule)

	return (p1, p2)


test = """\
Alice would gain 54 happiness units by sitting next to Bob.
Alice would lose 79 happiness units by sitting next to Carol.
Alice would lose 2 happiness units by sitting next to David.
Bob would gain 83 happiness units by sitting next to Alice.
Bob would lose 7 happiness units by sitting next to Carol.
Bob would lose 63 happiness units by sitting next to David.
Carol would lose 62 happiness units by sitting next to Alice.
Carol would gain 60 happiness units by sitting next to Bob.
Carol would gain 55 happiness units by sitting next to David.
David would gain 46 happiness units by sitting next to Alice.
David would lose 7 happiness units by sitting next to Bob.
David would gain 41 happiness units by sitting next to Carol.
"""
expected_rule: Rule = {
	"Alice": {"Bob": 54, "Carol": -79, "David": -2},
	"Bob": {"Alice": 83, "Carol": -7, "David": -63},
	"Carol": {"Alice": -62, "Bob": 60, "David": 55},
	"David": {"Alice": 46, "Bob": -7, "Carol": 41},
}
got_rule = build_rule(test)
assert expected_rule == got_rule, (
	f"build_rule test failed: expected {expected_rule}, got {got_rule}"
)

expected = (330, 286)
got = solve(test)
assert expected[0] == got[0], (
	f"part 1 test failed: expected {expected[0]}, got {got[0]}"
)
assert expected[1] == got[1], (
	f"part 2 test failed: expected {expected[1]}, got {got[1]}"
)


sys.path.append("../..")
from get_data import get_data

p1, p2 = solve(get_data())
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
