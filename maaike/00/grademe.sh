#!/bin/bash

function test ()
{
	if [[ $# -ne 2 ]]; then; printf "\033[31;1mError:\033[0m\033[31mwrong number of arg in frunction\033[0m\n" 1>&2 && exit 1; fi
	echo "$2" | python *.py | cat -e > you.txt
	echo "$2" | cat -e > src.txt
	res=$(diff out.txt src.txt)
	printf "test\t$1\t"
	if [[ $? -eq 0 ]]
	then
		printf "\033[32;1mOK\033[0m\n"
		return 1
	else
		printf "
}
