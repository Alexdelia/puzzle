dart_escaped() {
	local quote=$1 text=${2//\\/\\\\}
	text=${text//$quote/\\$quote}
	text=${text//\$/\\\$}
	text=${text//$'\r'/\\r}
	printf '%s%s%s' "$quote" "${text//$'\n'/\\n}" "$quote"
}

dart_raw() {
	local quote=$1 text=$2
	[[ $text != *$quote* && $text != *[$'\n\r']* ]] || return 0
	printf 'r%s%s%s' "$quote" "$text" "$quote"
}

dart_raw_triple() {
	local quote=$1 text=$2
	[[ $text != *$quote$quote$quote* && $text != *$quote && ! $text =~ ^[[:blank:]]*$'\n' ]] || return 0
	printf 'r%s%s%s' "$quote$quote$quote" "$text" "$quote$quote$quote"
}

dart_wrapper() {
	local literal=$1 executable=$2 arguments= argument
	shift 2
	for argument in "$@"; do
		arguments+="'$argument',"
	done
	printf "import'dart:io';main()=>Process.start('%s',[%s%s],mode:ProcessStartMode.values[1]);" "$executable" "$arguments" "$literal"
}

emit_dart() {
	local literal
	for literal in "$(dart_raw "'" "$core_body")" "$(dart_raw '"' "$core_body")" \
		"$(dart_raw_triple "'" "$core_body")" "$(dart_raw_triple '"' "$core_body")" \
		"$(dart_escaped "'" "$core_body")" "$(dart_escaped '"' "$core_body")"; do
		[[ -n $literal ]] && emit dart "$(dart_wrapper "$literal" "$@")"
	done
	return 0
}
