#!/usr/bin/env python3


from tqdm import tqdm

from src.answer import DECODE, ENCODE, b10tob, btob10

assert len(ENCODE) == len(set(ENCODE)), "ENCODE must be unique"
assert len(DECODE) == len(set(DECODE)), "DECODE must be unique"

RESULT = ".2048_results.out"
ANSWER = "./src/answer.py"


def _check_encoding(n: str, encode: str):
	F_N = "n.log"
	F_ENCODE = "encode.log"
	F_DECODE = "decode.log"

	decode = b10tob(btob10(encode, ENCODE), DECODE)

	if n == decode:
		return

	print("encoding failed", flush=True)
	print("len(n) =", len(n), flush=True)
	print("len(encode) =", len(encode), flush=True)
	print("len(decode) =", len(decode), flush=True)

	with open(F_N, "w") as f:
		f.write(n)
	with open(F_ENCODE, "w") as f:
		f.write(encode)
	with open(F_DECODE, "w") as f:
		f.write(decode)

	print("log files written to", [F_N, F_ENCODE, F_DECODE], flush=True)

	raise Exception("encoding failed")


def encode() -> dict[str, str]:
	print("reading results from", RESULT, flush=True)

	f = open(RESULT, "r")
	t = sum([1 for _ in f])
	f.close()

	f = open(RESULT, "r")
	out = {}

	print(f"encoding results from b{len(DECODE)} to b{len(ENCODE)}", flush=True)

	p = tqdm(total=t)
	for line in f:
		l = line.split()
		p.write(f"seed: \033[33;1m{l[0]}\033[0m")

		n = DECODE[1] + l[-1]
		b = b10tob(btob10(n, DECODE), ENCODE)

		p.write(
			f"  '->\t\033[35;1m{len(n)}\033[0m\
 -> \033[36;1m{len(b)}\033[0m\
\tcompressed to \033[32;1m{len(b) / len(n) * 100:.3f}%\033[0m"
		)

		out[int(l[1])] = b

		p.write("checking ...")
		_check_encoding(n, b)
		p.write("  '->\t\033[32;1mOK\033[0m")

		p.update(1)
	p.close()

	f.close()

	return out


def write(out: dict[str, str]):
	print("writing results to", ANSWER, flush=True)

	start = r"    answer = "
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


if __name__ == "__main__":
	write(encode())
