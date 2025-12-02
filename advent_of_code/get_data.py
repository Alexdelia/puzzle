import sys

if __name__ == "__main__":
	print(
		f"\033[35;1m{__file__}\033[0m is supposed to be \033[32;1mimported\033[0m as a \033[32;1mmodule\033[0m"  # noqa: E501
	)
	# sysexits.h `EX_USAGE` https://github.com/openbsd/src/blob/master/include/sysexits.h#L101
	sys.exit(64)


import re
from pathlib import Path

from aocd import get_data as aocd_get_data

import __main__


def get_data() -> str:
	if hasattr(__main__, "__file__") is False:
		raise ValueError("could not determine day and year from interactive session")

	dirs = Path(__main__.__file__).resolve().parents

	day = 0
	year = 0

	dirs_len = len(dirs)
	index = 0

	while index < dirs_len and day == 0:
		if match := re.match(r"^d(\d{2})$", dirs[index].name):
			day = int(match.group(1))
		index += 1

	while index < dirs_len and year == 0:
		if match := re.match(r"^y(\d{4})$", dirs[index].name):
			year = int(match.group(1))
		index += 1

	if day == 0 or year == 0:
		raise ValueError(
			f"could not determine day and year from path '{__main__.__file__}'"
		)

	return aocd_get_data(day=day, year=year)
