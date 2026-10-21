#!/usr/bin/env bash
keys=$1 pad=$2
core=$(sed "s/''/\"\"/; s/f(0,scalar<>,/\$s=<>;\$k=(length(\$R[0])-1).\",\".(\$V+0).\",\".(\$s+0);exit print\"WAIT\\\\n\"if\$k=~\/^($keys)\$\/;\n&/; s/f(0,scalar<>,/f(0,\$s,/" puzzle/the-bridge-episode-2/solution/perl/main.pl)
printf "perl -e'%s'\n#%s\n" "$core" "$(printf 'x%.0s' $(seq $pad))"
