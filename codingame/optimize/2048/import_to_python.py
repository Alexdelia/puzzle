#!/usr/bin/env python3

from src.answer import DECODE, ENCODE, b10tob, btob10

RESULT = ".2048_results.out"
ANSWER = "./src/answer.py"

print("reading results from", RESULT, flush=True)
f = open(RESULT, "r")
out = {}

print("encoding results:", flush=True)
for i, line in enumerate(f):
    print(f"\r{i}", end="")
    l = line.split()
    n = l[-1]
    b = b10tob(btob10(n, DECODE), ENCODE)
    out[int(l[1])] = b
    assert b10tob(btob10(b, ENCODE), DECODE) == n
f.close()
print()

start = r"answer = "
line = start + str(out) + "\n"

f = open(ANSWER, "r")
lines = f.readlines()
f.close()

for i, l in enumerate(lines):
    if l.startswith(start):
        lines[i] = line
        break

f = open(ANSWER, "w")
f.writelines(lines)
f.close()
