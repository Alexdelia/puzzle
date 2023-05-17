C, M, Y, K = map(float, (input() for _ in range(4)))

R = int(255 * (1 - C) * (1 - K) + 0.5)
G = int(255 * (1 - M) * (1 - K) + 0.5)
B = int(255 * (1 - Y) * (1 - K) + 0.5)

print(f"{R},{G},{B}")