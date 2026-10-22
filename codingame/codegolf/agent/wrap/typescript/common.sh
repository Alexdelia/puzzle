ts_double_quoted() {
	local text
	text=$(escape_c_string "$1")
	printf '"%s"' "${text//$'\r'/\\r}"
}

ts_single_quoted() {
	local text=${1//\\/\\\\}
	text=${text//\'/\\\'}
	text=${text//$'\n'/\\n}
	printf "'%s'" "${text//$'\r'/\\r}"
}

ts_template() {
	local text=${1//\\/\\\\}
	text=${text//'`'/'\`'}
	printf '`%s`' "${text//'${'/'\${'}"
}

ts_literal() {
	local best candidate
	best=$(ts_double_quoted "$1")
	candidate=$(ts_single_quoted "$1")
	((${#candidate} < ${#best})) && best=$candidate
	if [[ $1 != *$'\r'* ]]; then
		candidate=$(ts_template "$1")
		((${#candidate} < ${#best})) && best=$candidate
		if [[ $1 != *'`'* && $1 != *'${'* && $1 != *\\ ]]; then
			candidate=String.raw\`$1\`
			((${#candidate} < ${#best})) && best=$candidate
		fi
	fi
	printf '%s' "$best"
}

ts_exec() {
	printf 'require("child_process").execSync(%s,{stdio:[0,1]})' "$(ts_literal "$1")"
}

ts_spawn() {
	local program=$1 arguments= argument
	shift
	for argument in "$@"; do
		arguments+=${arguments:+,}$(ts_literal "$argument")
	done
	printf 'require("child_process").spawnSync("%s",[%s],{stdio:[0,1]})' "$program" "$arguments"
}
