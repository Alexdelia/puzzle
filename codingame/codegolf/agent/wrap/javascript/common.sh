escape_template() {
	local text=${1//\\/\\\\}
	text=${text//\`/\\\`}
	printf '%s' "${text//\$\{/\\\$\{}"
}

exec_sync() {
	printf 'require("child_process").execSync(`%s`,{stdio:[0,1]})' "$(escape_template "$1")"
}

exec_file_sync() {
	local interpreter=$1 arguments= argument
	shift
	for argument in "${@:1:$#-1}"; do
		arguments+="\"$(escape_double_quoted "$argument")\","
	done
	printf 'require("child_process").execFileSync("%s",[%s`%s`],{stdio:[0,1]})' "$interpreter" "$arguments" "$(escape_template "${!#}")"
}

emit_interpreter() {
	emit js "$(exec_sync "$(shell_command "$@")")"
	emit js "$(exec_file_sync "$@" "$core_body")"
}
