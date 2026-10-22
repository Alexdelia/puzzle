pascal_string() {
	local text=${1//\'/\'\'}
	text=${text//$'\n'/\'#10\'}
	text=${text//$'\r'/\'#13\'}
	printf "'%s'" "$text"
}

pascal_arguments() {
	local arguments= argument
	for argument in "$@"; do
		arguments+=,$(pascal_string "$argument")
	done
	printf '%s' "${arguments#,}"
}

emit_pascal() {
	emit pas "uses unix;begin $1end."
}

emit_exec() {
	local interpreter=$1
	shift
	local -a separate=("$@" "$core_body") attached=("$@")
	attached[-1]+=$core_body
	emit_pascal "fpexeclp($(pascal_string "$interpreter"),[$(pascal_arguments "${attached[@]}")])"
	emit_pascal "fpexeclp($(pascal_string "$interpreter"),[$(pascal_arguments "${separate[@]}")])"
}

emit_system() {
	emit_pascal "fpsystem($(pascal_string "$1"))"
}
