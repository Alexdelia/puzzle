fsharp_regular() {
	local text=$1 out= index char next
	for ((index = 0; index < ${#text}; index++)); do
		char=${text:index:1}
		next=${text:index+1:1}
		case $char in
		'"') out+='\"' ;;
		\\) [[ -z $next || $next == [ntbrafv\\\"\'0-9xuU$'\n'] ]] && out+='\\' || out+='\' ;;
		*) out+=$char ;;
		esac
	done
	printf '"%s"' "$out"
}

fsharp_verbatim() {
	printf '@"%s"' "${1//\"/\"\"}"
}

fsharp_triple() {
	[[ $1 != *'"""'* && $1 != '"'* && $1 != *'"' ]] || return 0
	printf '"""%s"""' "$1"
}

fsharp_literal() {
	local best candidate
	best=$(fsharp_regular "$1")
	for candidate in "$(fsharp_verbatim "$1")" "$(fsharp_triple "$1")"; do
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

fsharp_start() {
	local call="System.Diagnostics.Process.Start(\"$1\",$2)"
	[[ $kind == interactive ]] && call+=.WaitForExit\(\)
	printf '%s' "$call"
}

emit_interpreter() {
	local interpreter=$1 argument line= list=
	shift
	for argument in "$@" "$core_body"; do
		line+=" $(dotnet_argument "$argument")"
		list+=";$(fsharp_literal "$argument")"
	done
	emit fs "$(fsharp_start "$interpreter" "$(fsharp_literal "${line# }")")"
	emit fs "$(fsharp_start "$interpreter" "[${list#;}]")"
}
