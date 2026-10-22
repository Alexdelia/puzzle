ocaml_minimal_escape() {
	local text=$1 escaped= index character next
	for ((index = 0; index < ${#text}; index++)); do
		character=${text:index:1}
		next=${text:index+1:1}
		if [[ $character == '"' ]]; then
			escaped+='\"'
		elif [[ $character == '\' && ( -z $next || $next == [\\\"\'ntbr\ 0-9xou$'\n'] ) ]]; then
			escaped+='\\'
		else
			escaped+=$character
		fi
	done
	printf '%s' "$escaped"
}

ocaml_quoted_delimiter() {
	local id
	for id in '' a b c d e; do
		[[ $1 != *"|$id}"* ]] && { printf '%s' "$id"; return; }
	done
	return 1
}

emit_ocaml_command() {
	local command=$1 id escaped minimal
	escaped=$(escape_double_quoted "$command")
	minimal=$(ocaml_minimal_escape "$command")
	emit ml "Sys.command\"$escaped\""
	[[ $minimal == "$escaped" ]] || emit ml "Sys.command\"$minimal\""
	id=$(ocaml_quoted_delimiter "$command") && emit ml "Sys.command{$id|$command|$id}"
	return 0
}
