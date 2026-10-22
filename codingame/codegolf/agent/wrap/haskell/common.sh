haskell_string() {
	local text
	text=$(escape_c_string "$1")
	text=${text//$'\t'/\\t}
	printf '"%s"' "${text//$'\r'/\\r}"
}

haskell_system() {
	printf 'import System.Cmd;main=system%s' "$(haskell_string "$1")"
}

haskell_raw_system() {
	local interpreter=$1 arguments= argument
	shift
	for argument in "$@"; do
		arguments+=,$(haskell_string "$argument")
	done
	printf 'import System.Cmd;main=rawSystem%s[%s]' "$(haskell_string "$interpreter")" "${arguments#,}"
}

emit_interpreter() {
	local interpreter=$1
	shift
	emit hs "$(haskell_system "$(shell_command "$interpreter" "$@")")"
	emit hs "$(haskell_raw_system "$interpreter" "$@" "$core_body")"
}
