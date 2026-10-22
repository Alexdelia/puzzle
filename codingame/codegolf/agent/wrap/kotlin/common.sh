kotlin_template_escaped() {
	local text=$1 replacement=$2 escaped=
	while [[ $text == *\$* ]]; do
		escaped+=${text%%\$*}
		text=${text#*\$}
		if [[ $text =~ ^[A-Za-z_\{] ]]; then
			escaped+=$replacement
		else
			escaped+=\$
		fi
	done
	printf '%s' "$escaped$text"
}

kotlin_quoted() {
	local text=${1//\\/\\\\}
	text=${text//\"/\\\"}
	text=$(kotlin_template_escaped "$text" '\$')
	text=${text//$'\r'/\\r}
	printf '"%s"' "${text//$'\n'/\\n}"
}

kotlin_raw() {
	[[ $1 != *'"""'* ]] || return 0
	printf '"""%s"""' "$(kotlin_template_escaped "$1" "\${'\$'}")"
}

kotlin_wrapper() {
	local literal=$1 arguments= argument
	shift
	for argument in "$@"; do
		arguments+="\"$argument\","
	done
	local call="ProcessBuilder($arguments$literal).inheritIO().start()"
	[[ $kind == interactive ]] && call+='.waitFor()'
	printf 'fun main(){%s}' "$call"
}

emit_kotlin() {
	local literal
	for literal in "$(kotlin_quoted "$core_body")" "$(kotlin_raw "$core_body")"; do
		[[ -n $literal ]] && emit kt "$(kotlin_wrapper "$literal" "$@")"
	done
	return 0
}
