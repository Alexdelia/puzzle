#!/usr/bin/env python3

RESULT = ".2048_results.out"
ANSWER = "./src/answer.rs"

f = open(RESULT, "r")
out = []
for line in f:
    l = line.split()
    out.append((int(l[1]), l[-1]))
f.close()

start = r"    let d: HashMap<Seed, &str> = HashMap::from("
line = start + str(out).replace("'", "\"") + ");\n"

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
