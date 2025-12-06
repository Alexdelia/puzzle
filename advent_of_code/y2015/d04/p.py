#!/usr/bin/env python3

import re
from hashlib import md5
from pathlib import Path

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", Path(__file__).parent.name))
YEAR = int(re.sub(r"[^0-9]", "", Path(__file__).parent.parent.name))
DATA: str = get_data(day=DAY, year=YEAR)

i = 1

while md5(f"{DATA}{i}".encode()).hexdigest()[:5] != "00000":  # noqa: S324
	i += 1

print(f"part 1:\t{i}")

while md5(f"{DATA}{i}".encode()).hexdigest()[:6] != "000000":  # noqa: S324
	i += 1

print(f"part 2:\t{i}")
