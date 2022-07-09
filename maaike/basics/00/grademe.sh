#!/bin/bash

clr="\033[0m"
bld="\033[1m"
ita="\033[3m"
gre="\033[32m"
red="\033[31m"
mag="\033[35m"

if [[ $# -ne 1 ]]
then
	printf "usage:\n$bld./grademe.sh$mag$ita file_to_test$clr\n"
	exit 1
fi

sep="==============================================================================================="
ok="✔"
ko="⤫"

total=0
pass=0

rm -f log.txt

function test ()
{
	if [[ $# -ne 3 ]]
	then
		printf "$red${bld}Error:$clr$red\twrong number of arg in frunction$clr\n" 1>&2
		exit 1
	fi
	printf "${bld}test\t$1\t"
	printf "$2" | ./$prog | cat -e &> you.txt
	printf "$3" | cat -e > src.txt
	res=$(diff you.txt src.txt)
	if [[ $? -eq 0 ]]
	then
		printf "$gre$ok$clr\n"
		((pass=pass+1))
	else
		printf "$red$ko$clr\n"
		printf "test $1 KO:\n" >> log.txt
		printf "  diff:\n" >> log.txt
		printf "$res\n\n" >> log.txt
		printf "$sep\n\n" >> log.txt
		printf "  your output:\n" >> log.txt
		cat you.txt >> log.txt
		printf "\n$sep\n\n" >> log.txt
		printf "  source output:\n" >> log.txt
		cat src.txt >> log.txt
		printf "\n$sep\n\n\n" >> log.txt
	fi
	rm you.txt src.txt
	((total=total+1))
}

test 0 "" ""
test 1 "Hello world!" "Hello world!"
test 2 "42" "42"
test 3 "hey\nyou" "hey"
test 4 "that is\ta tab" "that is\ta tab"
test 5 "well done 😊  🧑🏻‍💻" "well done 😊  🧑🏻‍💻"

printf "\n$bld$gre$pass$clr$bld/$total tests\t["
if [[ $pass -ne $total ]]
then
	printf "${red}KO$clr]\n"
	printf "${bld}look at ${mag}log.txt$clr$bld to see the fails\n"
	exit 1
else
	printf "${gre}OK$clr]\n"
	exit 0
fi
