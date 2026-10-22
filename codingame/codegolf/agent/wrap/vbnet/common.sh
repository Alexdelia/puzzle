vbnet_literal() {
	printf '"%s"' "${1//\"/\"\"}"
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

vbnet_module() {
	printf 'Module A\nSub Main\n%s\nEnd Sub\nEnd Module' "$1"
}

emit_start() {
	local call="Process.Start(\"$1\",$2)"
	emit vb "$(vbnet_module "$call.WaitForExit")"
	[[ $kind == interactive ]] || emit vb "$(vbnet_module "$call")"
}

emit_interpreter() {
	local interpreter=$1 argument line= list=
	shift
	for argument in "$@" "$core_body"; do
		line+=" $(dotnet_argument "$argument")"
		list+=",$(vbnet_literal "$argument")"
	done
	emit_start "$interpreter" "$(vbnet_literal "${line# }")"
	emit_start "$interpreter" "{${list#,}}"
}
