swift_escaped() {
	local text=${1//\\/\\\\}
	text=${text//\"/\\\"}
	text=${text//$'\r'/\\r}
	printf '"%s"' "${text//$'\n'/\\n}"
}

swift_raw() {
	[[ $1 != *'"#'* && $1 != *'\#'* ]] || return 0
	local text=${1//$'\r'/\\#r}
	printf '#"%s"#' "${text//$'\n'/\\#n}"
}

swift_raw_multiline() {
	[[ $1 == *$'\n'* && $1 != *'"""#'* && $1 != *'\#'* && $1 != *$'\r'* ]] || return 0
	printf '#"""\n%s\n"""#' "$1"
}

escape_shell_double_quoted() {
	local text=${1//\\/\\\\}
	text=${text//\"/\\\"}
	text=${text//\$/\\\$}
	printf '%s' "${text//\`/\\\`}"
}

shell_double_command() {
	local interpreter=$1
	shift
	local command=$interpreter argument
	for argument in "$@"; do
		command+=" $argument"
	done
	[[ $interpreter == bash || $interpreter == sh ]] && command+=" "
	printf '%s"%s"' "$command" "$(escape_shell_double_quoted "$core_body")"
}

emit_swift() {
	local literal
	for literal in "$(swift_escaped "$1")" "$(swift_raw "$1")" "$(swift_raw_multiline "$1")"; do
		[[ -n $literal ]] && emit swift "import Glibc;system($literal)"
	done
	return 0
}

emit_swift_command() {
	emit_swift "$(shell_command "$@")"
	emit_swift "$(shell_double_command "$@")"
}
