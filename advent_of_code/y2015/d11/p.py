#!/usr/bin/env python3

import sys


def is_valid_password(pwd: str) -> bool:
	if not any(
		ord(pwd[i]) + 1 == ord(pwd[i + 1]) and ord(pwd[i]) + 2 == ord(pwd[i + 2])
		for i in range(len(pwd) - 2)
	):
		return False

	pairs: set[str] = set()
	i = 0
	while i < len(pwd) - 1:
		if pwd[i] in "iol":
			return False

		if pwd[i] == pwd[i + 1]:
			pairs.add(pwd[i])
			i += 2
		else:
			i += 1

	return len(pairs) >= 2


def increment_password(pwd: str) -> str:
	res = list(pwd)

	i = len(pwd) - 1
	while i >= 0:
		c = res[i]

		if c == "z":
			res[i] = "a"
			i -= 1
			continue

		res[i] = chr(ord(c) + 1)
		if res[i] in "iol":
			res[i] = chr(ord(res[i]) + 1)
		break

	return "".join(res)


def solve(data: str) -> tuple[str, int]:
	pwd = data.strip()

	pwd = increment_password(pwd)
	while not is_valid_password(pwd):
		pwd = increment_password(pwd)

	p1 = pwd

	pwd = increment_password(pwd)
	while not is_valid_password(pwd):
		pwd = increment_password(pwd)

	p2 = pwd

	return (p1, p2)


invalid_pwd = [
	"hijklmmn",
	"abbceffg",
	"abbcegjk",
]
for pwd in invalid_pwd:
	assert not is_valid_password(pwd), f"invalid password test failed: {pwd}"

valid_pwd = [
	"abcdffaa",
	"ghjaabcc",
]
for pwd in valid_pwd:
	assert is_valid_password(pwd), f"valid password test failed: {pwd}"

test = [
	("abcdefgh", "abcdffaa"),
	("ghijklmn", "ghjaabcc"),
]
for data, expected in test:
	got = solve(data)
	assert expected == got[0], f"part 1 test failed: expected {expected}, got {got[0]}"


sys.path.append("../..")
from get_data import get_data

p1, p2 = solve(get_data())
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
