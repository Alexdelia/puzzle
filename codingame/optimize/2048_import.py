#!/usr/bin/env python3

FILE = ".2048_results.out"
f = open(FILE, "r")
out = []
for line in f:
    l = line.split()
    out.append((int(l[1]), l[-1]))
f.close()
print(str(out).replace("'", "\""))
