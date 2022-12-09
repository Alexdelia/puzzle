import math
import sys

SIZE = 4
CHUNK = 9000


def get_input():
    seed = int(input())
    score = int(input())
    grid = []
    for _ in range(SIZE):
        grid.append(input().split())
    return seed, score, grid

answer = {}

seed, _, _ = get_input()

if seed in answer:
    m = answer[seed]
    for i in range(0, len(m), CHUNK):
        print(m[i:i + CHUNK])
        get_input()

# while True:
#     print("LURU")
#     get_input()
