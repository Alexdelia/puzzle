N = 1000

n = d = 1
np = dp = 10

t = 0
for k in range(N):
	n, d = n + 2 * d, n + d

	if n >= np:
		np *= 10
	if d >= dp:
		dp *= 10

	if np > dp:
		t += 1

print(t)
