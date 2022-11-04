import math
import string
import sys
from typing import Dict, List

d: Dict[str, List[str]] = {'?': []}

w = int(input())
h = int(input())
t = input()
for i in range(h):
    row = input()
    for a in range(26):
        if string.ascii_uppercase[a] not in d:
            d[string.ascii_uppercase[a]] = []
        d[string.ascii_uppercase[a]].append(row[a * w:(a + 1) * w])
    # getting question mark character
    d["?"].append(row[26 * w:])

for i in range(h):
    for c in t.upper():
        if c in string.ascii_uppercase:
            print(d[c][i], end="")
        else:
            print(d['?'][i], end="")
    print()
