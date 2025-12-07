#!/usr/bin/env python3

from pathlib import Path

from src.answer import DECODE, ENCODE, b10tob, btob10
from tqdm import tqdm

assert len(ENCODE) == len(set(ENCODE)), "ENCODE must be unique"
assert len(DECODE) == len(set(DECODE)), "DECODE must be unique"

RESULT = ".2048_results.out"
ANSWER = "./src/answer.py"


def _check_encoding(n: str, encode: str) -> None:
	f_n = "n.log"
	f_encode = "encode.log"
	f_decode = "decode.log"

	decode = b10tob(btob10(encode, ENCODE), DECODE)

	if n == decode:
		return

	print("encoding failed", flush=True)
	print("len(n) =", len(n), flush=True)
	print("len(encode) =", len(encode), flush=True)
	print("len(decode) =", len(decode), flush=True)

	with Path.open(f_n, "w") as f:
		f.write(n)
	with Path.open(f_encode, "w") as f:
		f.write(encode)
	with Path.open(f_decode, "w") as f:
		f.write(decode)

	print("log files written to", [f_n, f_encode, f_decode], flush=True)

	raise Exception("encoding failed")


def encode() -> dict[str, str]:
	print("reading results from", RESULT, flush=True)

	f = Path.open(RESULT)
	t = sum([1 for _ in f])
	f.close()

	f = Path.open(RESULT)
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


def write(out: dict[str, str]) -> None:
	print("writing results to", ANSWER, flush=True)

	start = r"    answer = "
	line = start + str(out) + "\n"

	f = Path.open(ANSWER)
	lines = f.readlines()
	f.close()

	for i, l in enumerate(lines):
		if l.startswith(start):
			lines[i] = line
			break

	f = Path.open(ANSWER, "w")
	f.writelines(lines)
	f.close()


if __name__ == "__main__":
	write(encode())
