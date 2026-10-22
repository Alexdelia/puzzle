csharp_regular() {
	local text=${1//\\/\\\\}
	text=${text//\"/\\\"}
	text=${text//$'\r'/\\r}
	printf '"%s"' "${text//$'\n'/\\n}"
}

csharp_verbatim() {
	printf '@"%s"' "${1//\"/\"\"}"
}

csharp_raw() {
	local text=$1 delimiter='"""'
	while [[ $text == *"$delimiter"* ]]; do
		delimiter+='"'
	done
	if [[ $text == *$'\n'* ]]; then
		printf '%s\n%s\n%s' "$delimiter" "$text" "$delimiter"
	elif [[ $text != '"'* && $text != *'"' ]]; then
		printf '%s%s%s' "$delimiter" "$text" "$delimiter"
	fi
}

csharp_literal() {
	local best candidate
	best=$(csharp_regular "$1")
	for candidate in "$(csharp_verbatim "$1")" "$(csharp_raw "$1")"; do
		[[ -n $candidate ]] && ((${#candidate} < ${#best})) && best=$candidate
	done
	printf '%s' "$best"
}

dotnet_argument() {
	local text=$1 out= index char run= quoted=
	[[ -z $text || $text == *[$' \t']* ]] && quoted=1
	for ((index = 0; index < ${#text}; index++)); do
		char=${text:index:1}
		case $char in
		\\) run+='\' && continue ;;
		'"') out+=$run$run'\"' ;;
		*) out+=$run$char ;;
		esac
		run=
	done
	if [[ -n $quoted ]]; then
		printf '"%s"' "$out$run$run"
	else
		printf '%s' "$out$run"
	fi
}

csharp_start() {
	local call="System.Diagnostics.Process.Start(\"$1\",$2)"
	[[ $kind == interactive ]] && call+=.WaitForExit\(\)
	printf '%s;' "$call"
}

emit_interpreter() {
	local interpreter=$1 argument line= list=
	shift
	for argument in "$@" "$core_body"; do
		line+=" $(dotnet_argument "$argument")"
		list+=",$(csharp_literal "$argument")"
	done
	emit cs "$(csharp_start "$interpreter" "$(csharp_literal "${line# }")")"
	emit cs "$(csharp_start "$interpreter" "[${list#,}]")"
}
