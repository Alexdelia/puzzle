cpp_escaped() {
	local text
	text=$(escape_c_string "$1")
	printf '"%s"' "${text//$'\r'/\\r}"
}

cpp_raw() {
	local delimiter
	for delimiter in '' x y z q; do
		[[ $1 == *")$delimiter\""* ]] && continue
		printf 'R"%s(%s)%s"' "$delimiter" "$1" "$delimiter"
		return 0
	done
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

emit_cpp() {
	local literal
	for literal in "$(cpp_escaped "$1")" "$(cpp_raw "$1")"; do
		[[ -n $literal ]] && emit cpp "#import<ios>
main(){system($literal);}"
	done
	return 0
}

emit_cpp_command() {
	emit_cpp "$(shell_command "$@")"
	emit_cpp "$(shell_double_command "$@")"
}
