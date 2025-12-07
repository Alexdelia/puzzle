import re


def expression_index(n: int, ex: list[str]) -> int | None:
	for i, e in enumerate(ex):
		if n == eval(e):  # noqa: S307
			return i
	return None


r = expression_index(int(input()), re.sub(r"[()]", " ", input()).split())

print(["none", f"index {r}"][r is not None])
