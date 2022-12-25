#!/usr/bin/env python3

import re
from os.path import dirname
from typing import Any, Callable

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

DATA_EXAMPLE = """\
1=-0-2
12111
2=0=
21
2=01
111
20012
112
1=-1=
1-12
12
1=
122\
"""


def snafu_to_b10(snafu: str) -> int:
    snafu = snafu[::-1]
    t = 0

    for i in range(len(snafu)):
        if snafu[i].isdigit():
            n = int(snafu[i])
        elif snafu[i] == '-':
            n = -1
        elif snafu[i] == '=':
            n = -2
        else:
            raise ValueError(f"Invalid character '{snafu[i]}'")
        
        t += (5 ** i) * n

    return t


def b10_to_snafu(n: int) -> str:
    if n == 0:
        return ''

    return b10_to_snafu((n + 2) // 5) + "=-012"[(n + 2) % 5]


def check(f: Callable, a: Any, expected: Any):
    assert f(a) == expected, f"\
\033[35;1m{a}\033[0m -> \
\033[31;1m{f(a)}\033[0m \
\033[33;1m!=\033[0m \
\033[32;1m{expected}\033[0m"


check(snafu_to_b10, "1", 1)
check(snafu_to_b10, "2", 2)
check(snafu_to_b10, "1=", 3)
check(snafu_to_b10, "1-", 4)
check(snafu_to_b10, "10", 5)
check(snafu_to_b10, "1121-1110-1=0", 314159265)

check(b10_to_snafu, 1, "1")
check(b10_to_snafu, 2, "2")
check(b10_to_snafu, 3, "1=")
check(b10_to_snafu, 4, "1-")
check(b10_to_snafu, 5, "10")
check(b10_to_snafu, 314159265, "1121-1110-1=0")

assert sum(snafu_to_b10(snafu) for snafu in DATA_EXAMPLE.splitlines()) == 4890
assert b10_to_snafu(4890) == "2=-1=0"
