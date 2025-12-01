SIZE = 4
CHUNK = 9000
ENCODE = r"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!#$%&'()*+-./:;<=>?@[]^_`{|}~"
DECODE = r"UDLR"


def btob10(n: str, key: str) -> int:
	ret: int = 0
	l: int = len(key)

	for i, c in enumerate(n[::-1]):
		ret += key.index(c) * (l**i)

	return ret


def b10tob(n: int, key: str) -> str:
	ret: str = ""
	l: int = len(key)

	while n > 0:
		n, r = divmod(n, l)
		ret += key[r]

	return ret[::-1]


def get_input():
	seed = int(input())
	score = int(input())
	grid = []
	for _ in range(SIZE):
		grid.append(input().split())
	return seed, score, grid


if __name__ == "__main__":
	answer = {}

	seed, _, _ = get_input()

	if seed in answer:
		m = b10tob(btob10(answer[seed], ENCODE), DECODE)
		for i in range(0, len(m), CHUNK):
			print(m[i : i + CHUNK])
			get_input()

	# while True:
	#     print("LURU")
	#     get_input()
