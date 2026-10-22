escape_shell_double_quoted() {
	local text=${1//\\/\\\\}
	text=${text//\"/\\\"}
	text=${text//\$/\\\$}
	printf '%s' "${text//\`/\\\`}"
}

objc_system() {
	printf 'main(){system("%s");}' "$(escape_c_string "$1")"
}

emit_system() {
	local command=$1 argument
	shift
	for argument in "$@"; do
		command+=" $argument"
	done
	emit m "$(objc_system "$(shell_command "$command")")"
	[[ $command == *' -'* ]] || command+=" "
	emit m "$(objc_system "$command\"$(escape_shell_double_quoted "$core_body")\"")"
}
