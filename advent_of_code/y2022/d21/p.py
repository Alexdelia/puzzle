#!/usr/bin/env python3

import re
from copy import deepcopy
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

DATA_EXAMPLE = """root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32
"""


def parse(data: str) -> dict[str, int | str]:
	d: dict[str, int | str] = {}

	for l in data.splitlines():
		name, value = l.split(": ")
		d[name] = int(value) if value.isnumeric() else value

	return d


def get_value(d: dict[str, int | str], name: str) -> int:
	try:
		value = d[name]
	except KeyError:
		print(f"key {name} not found")
		return 0

	if isinstance(value, int):
		return value

	n1, op, n2 = value.split(" ")
	n = int(eval(f"{get_value(d, n1)} {op} {get_value(d, n2)}"))
	d[name] = n
	return n


def use_name(d: dict[str, int | str], root: str, name: str) -> bool:
	value = d[root]

	if isinstance(value, int):
		return False

	n1, _, n2 = value.split(" ")
	if n1 == name or n2 == name:
		return True

	return use_name(d, n1, name) or use_name(d, n2, name)


def find_branch(d: dict[str, int | str], root: str, name: str) -> tuple[str, str]:
	r = d[root]
	assert isinstance(r, str)

	n1, _, n2 = r.split(" ")
	if n1 == name:
		return (name, n2)
	if n2 == name:
		return (name, n1)

	ret = []
	nret = []
	if use_name(d, n1, name):
		ret.append(n1)
	else:
		nret.append(n1)

	if use_name(d, n2, name):
		ret.append(n2)
	else:
		nret.append(n2)

	assert len(ret) == 1, f"{len(ret)} branch found for {name} in {root}"
	assert len(nret) == 1, f"{len(nret)} branch found for no {name} in {root}"
	return (ret[0], nret[0])


def equation(d: dict[str, int | str], root: str, name: str) -> str:
	if root == name:
		return f"({name})"

	val = d[root]

	if isinstance(val, int):
		return str(val)

	n1, op, n2 = val.split(" ")

	b = find_branch(deepcopy(d), root, name)

	if b[0] == n1:
		return f"({equation(d, b[0], name)} {op} {get_value(d, b[1])})"
	elif b[0] == n2:
		return f"({get_value(d, b[1])} {op} {equation(d, b[0], name)})"
	else:
		raise Exception(f"branch not found for {name} in {root}")


assert get_value(parse(DATA_EXAMPLE), "root") == 152
print(f"part 1:\t{int(get_value(parse(DATA), 'root'))}")

d = parse(DATA_EXAMPLE)
b = find_branch(deepcopy(d), "root", "humn")
e = equation(d, b[0], "humn").replace("humn", "301")
assert int(eval(e)) == get_value(d, b[1])

assert DATA.count("humn") == 2
d = parse(DATA)
b = find_branch(deepcopy(d), "root", "humn")
e = equation(d, b[0], "humn")
x = get_value(d, b[1])
print("part 2:\ttoo big in wolfram alpha, but not in mathpapa.com :")
print(f"{x} = {e.replace('humn', 'x')}")
