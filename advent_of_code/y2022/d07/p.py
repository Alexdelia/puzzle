#!/usr/bin/env python3

from __future__ import annotations

import re
from os.path import dirname
from typing import Dict, List

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

P1_SIZE = 100000
P2_TOTAL = 70000000
P2_UPDATE = 30000000


class File:
    def __init__(self, name: str, size: int):
        self.name: str = name
        self.size: int = size
    
    def __repr__(self):
        return f"{self.name}\t{self.size}"
    
    def __str__(self):
        return f"{self.name}\t{self.size}"

class Dir:
    def __init__(self, name: str, parent: Dir):
        self.name: str = name
        self.parent: Dir = parent
        self.dir: Dict[str, Dir] = {}
        self.file: List[File] = []
        self.size: int = 0
    
    def __repr__(self):
        return f"{self.name}\t{self.size}:\n\t{self.dir}\t{self.file}"

    def __str__(self):
        return f"{self.name}\t{self.size}:\n\t{self.dir}\t{self.file}"
    
    def lsize(self) -> int:
        self.size = sum(f.size for f in self.file)
        self.size += sum(v.lsize() for v in self.dir.values())
        return self.size

root = Dir("/", None)
assert lines[0] == "$ cd /"

pwd: Dir = root
i = 1
while i < len(lines):
    if lines[i].startswith("$"):
        _, cmd, *args = lines[i].split()
        if cmd == "cd":
            assert len(args) == 1
            if args[0] == "..":
                pwd = pwd.parent
            else:
                pwd = pwd.dir[args[0]]
        elif cmd == "ls":
            assert len(args) == 0
            i += 1
            while i < len(lines):
                if lines[i].startswith("$"):
                    i -= 1
                    break
                if lines[i].startswith("d"):
                    _, name = lines[i].split()
                    pwd.dir[name] = Dir(name, pwd)
                else:
                    size, name = lines[i].split()
                    pwd.file.append(File(name, int(size)))
                i += 1
    i += 1

s = root.lsize()
print(f"total size:\t{s}")
assert s == root.size

# find all dir with size < SIZE
def find_small_dir(root: Dir) -> List[Dir]:
    dirs = []
    if root.size < P1_SIZE:
        dirs.append(root)
    for d in root.dir.values():
        dirs.extend(find_small_dir(d))
    return dirs

dirs = find_small_dir(root)

t = 0
for d in dirs:
    t += d.size
print(f"part 1:\t{t}")

unused = P2_TOTAL - root.size
print(f"unused:\t{unused}")

# find smallest dir with size + unused > P2_UPDATE
def find_big_dir(root: Dir, dir_to_beat: Dir) -> Dir:
    if root.size + unused > P2_UPDATE and root.size < dir_to_beat.size:
        dir_to_beat = root
    for d in root.dir.values():
        dir_to_beat = find_big_dir(d, dir_to_beat)
    return dir_to_beat

dir = find_big_dir(root, root)
print(f"part 2:\t{dir.size}")
