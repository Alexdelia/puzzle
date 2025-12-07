import os
import re
import sys

if __name__ == "__main__":
	print(
		f"\033[35;1m{__file__}\033[0m is supposed to be \033[32;1mimported\033[0m as a \033[32;1mmodule\033[0m"
	)
	sys.exit(1)


def file(day: int | None = None) -> str:
	f = open(_match_file(day))
	content = f.read()
	f.close()
	return content


def _match_file(n: int | None = None) -> str:
	if len(sys.argv) == 2:
		return _find_file(sys.argv[1])

	if n:
		return _find_file(str(n))

	m = re.match(r"[./]*d[0-9]{2}/", sys.argv[0])
	if m:
		return _find_file(re.sub(r"[^0-9]", "", m.group(0)))
	else:
		print(f"usage:\t\033[1m{sys.argv[0]} \033[35m<input>\033[0m")
		sys.exit(1)


def _find_file(file: str) -> str:
	if os.path.isfile(file):
		return file

	try:
		nf = f"../input/d{int(file):02}.in"
		if os.path.isfile(nf):
			return nf
	except:
		pass

	print(f"\033[31;1m{file}\033[0m not found")
	sys.exit(1)
