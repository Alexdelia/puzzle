#!/usr/bin/env python3

RESULT = ".2048_results.out"

f = open(RESULT, "r")
print(sum([int(l.split()[-2]) for l in f]))
f.close()
