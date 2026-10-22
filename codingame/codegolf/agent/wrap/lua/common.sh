escape_lua_single_quoted() {
	local text=${1//\\/\\\\}
	text=${text//\'/\\\'}
	printf '%s' "${text//$'\n'/\\n}"
}

lua_long_bracket() {
	local text=$1 level=
	while [[ $text == *"]$level]"* || $text == *"]$level" ]]; do
		level+==
	done
	printf '[%s[%s]%s]' "$level" "$text" "$level"
}

emit_lua_execute() {
	local command=$1
	[[ $command == *$'\r'* ]] && return 0
	emit lua "os.execute\"$(escape_c_string "$command")\""
	emit lua "os.execute'$(escape_lua_single_quoted "$command")'"
	[[ $command == *$'\n'* ]] || emit lua "os.execute$(lua_long_bracket "$command")"
}

escape_shell_double_quoted() {
	local text=${1//\\/\\\\} character
	for character in '$' '`' '"'; do
		text=${text//"$character"/\\$character}
	done
	printf '%s' "$text"
}

shell_command_double_quoted() {
	local interpreter=$1
	shift
	local command=$interpreter argument
	for argument in "$@"; do
		command+=" $argument"
	done
	[[ $interpreter == bash || $interpreter == sh ]] && command+=" "
	printf '%s"%s"' "$command" "$(escape_shell_double_quoted "$core_body")"
}
