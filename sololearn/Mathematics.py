from typing import Optional

import re


def expression_index(n: int, ex: list[str]) -> Optional[int]:
	for i, e in enumerate(ex):
		if n == eval(e):
			return i
	return None


r = expression_index(int(input()), re.sub(r"[()]", " ", input()).split())

print(["none", f"index {r}"][r != None])
