#!/bin/bash

nb='^[0-9]+$'

if [[ "$#" < 1 ]]; then
	>&2 echo "No input"
elif [[ "$#" == 1 ]]; then
	Sentence_Formater -uwu "$1"
elif [[ "$#" == 2 ]]; then
	if [[ "$1" =~ $nb ]]; then
		Sentence_Formater -uwu "$1" "$2"
	else
		Sentence_Formater -uwu "$2" "$1"
	fi
fi	
