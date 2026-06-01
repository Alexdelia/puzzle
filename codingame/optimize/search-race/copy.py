#!/usr/bin/env python3

import base64, os, sys

script_dir = os.path.dirname(os.path.abspath(__file__))

validator_dir = os.path.join(script_dir, "validator")
output_dir = os.path.join(script_dir, "output")

SOLUTION_FILE_NAME = "solution.txt"
VALIDATOR_FILE_EXTENSION = ".txt"

TILT_OFFSET = 18

parts = []
for name in sorted(os.listdir(output_dir)):
    sol_path = os.path.join(output_dir, name, SOLUTION_FILE_NAME)
    flag_path = os.path.join(validator_dir, name + VALIDATOR_FILE_EXTENSION)
    if not os.path.isfile(sol_path):
        continue
    with open(flag_path) as f:
        flag = f.readline().rstrip("\n")
    buf = bytearray()
    with open(sol_path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            a, b = line.split()
            buf.append(int(a) + TILT_OFFSET)
            buf.append(int(b))
    compressed = base64.b64encode(buf).decode()
    parts.append('("' + flag + '","' + compressed + '"),')

print("".join(parts), end="")
