d_string() {
	if [[ $1 != *'`'* ]]; then
		printf '`%s`' "$1"
	elif [[ $1 != *'"'* ]]; then
		printf 'r"%s"' "$1"
	else
		printf '"%s"' "$(escape_double_quoted "$1")"
	fi
}

emit_d_call() {
	local call=$1 literal=$2
	emit d "import std;void main(){$literal.$call.wait;}"
	[[ $kind == interactive ]] || emit d "import std;void main(){$literal.$call;}"
}

emit_d_shell() {
	emit_d_call spawnShell "$(d_string "$(shell_command "$@")")"
}

emit_d_process() {
	local literals= argument
	for argument in "$@" "$core_body"; do
		literals+=,$(d_string "$argument")
	done
	emit_d_call spawnProcess "[${literals#,}]"
}

emit_d() {
	emit_d_shell "$@"
	emit_d_process "$@"
}
