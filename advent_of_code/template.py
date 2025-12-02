#!/usr/bin/env python3

import sys

sys.path.append("../..")
from get_data import get_data

DATA: str = get_data()

lines = DATA.splitlines()

print(lines)
