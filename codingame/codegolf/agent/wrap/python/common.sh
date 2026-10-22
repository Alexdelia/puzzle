python_escaped() {
	local text=${1//\\/\\\\} quote=$2
	text=${text//"$quote"/\\$quote}
	printf '%s' "${text//$'\n'/\\n}"
}

python_literals() {
	local text=$1 quote triple
	for quote in '"' "'"; do
		triple=$quote$quote$quote
		printf '%s\0' "$quote$(python_escaped "$text" "$quote")$quote"
		if [[ $text != *"$quote"* && $text != *$'\n'* && $text != *\\ ]]; then
			printf '%s\0' "r$quote$text$quote"
		fi
		if [[ $text != *"$triple"* && $text != *"$quote" ]]; then
			local backslashed=${text//\\/\\\\}
			[[ $backslashed != *"$quote" ]] && printf '%s\0' "$triple$backslashed$triple"
			[[ $text != *\\ ]] && printf '%s\0' "r$triple$text$triple"
		fi
	done
}

emit_system() {
	local literal
	while IFS= read -r -d '' literal; do
		emit py "import os;os.system($literal)"
	done < <(python_literals "$1")
}

emit_exec() {
	local interpreter=$1 arguments= argument literal
	shift
	local body=${!#}
	for argument in "${@:1:$#-1}"; do
		arguments+=",\"$argument\""
	done
	while IFS= read -r -d '' literal; do
		emit py "import os;os.execlp(\"$interpreter\",\"${interpreter:0:1}\"$arguments,$literal)"
	done < <(python_literals "$body")
}
