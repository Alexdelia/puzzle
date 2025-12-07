#!/usr/bin/env python3

from pathlib import Path

RESULT = ".2048_results.out"

f = Path.open(RESULT)
print(sum([int(l.split()[-2]) for l in f]))
f.close()
