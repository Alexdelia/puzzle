escape_shell_double_quoted() {
	local text=${1//\\/\\\\}
	text=${text//\"/\\\"}
	text=${text//\$/\\\$}
	printf '%s' "${text//\`/\\\`}"
}

c_system() {
	printf 'main(){system("%s");}' "$(escape_c_string "$1")"
}

emit_system() {
	local single_quoted double_quoted
	single_quoted=$(shell_command "$@")
	double_quoted=${single_quoted%%\'*}
	emit c "$(c_system "$single_quoted")"
	emit c "$(c_system "$double_quoted\"$(escape_shell_double_quoted "$core_body")\"")"
}
