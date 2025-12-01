import re

LEN = 7
RuleT = tuple[tuple[int, str]]
RULE: RuleT = ((2, r"[0-9]"), (2, r"[!@#$%&*]"))


def check(s: str, rule: RuleT) -> bool:
	return not any(len(re.findall(i[1], s)) < i[0] for i in rule)


s = input()

if len(s) < LEN or not check(s, RULE):
	print("Weak")
else:
	print("Strong")
