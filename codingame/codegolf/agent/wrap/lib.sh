core_file=${1:?usage: <core>.sh <core-file> <io|interactive> <out-dir>}
kind=${2:?usage: <core>.sh <core-file> <io|interactive> <out-dir>}
out_dir=${3:?usage: <core>.sh <core-file> <io|interactive> <out-dir>}
candidate_count=0

core_text=$(<"$core_file")
core_flags=
core_body=$core_text
if [[ $core_text == '#!'* ]]; then
	shebang=${core_text%%$'\n'*}
	read -r _ core_flags <<<"$shebang"
	core_body=${core_text#"$shebang"}
	core_body=${core_body#$'\n'}
fi

perl_args() {
	local words
	read -r -a words <<<"$core_flags"
	local last=$((${#words[@]} - 1))
	if ((last >= 0)) && [[ ${words[last]} =~ ^-[0-9alnpsw]*E$ ]]; then
		:
	elif ((last >= 0)) && [[ ${words[last]} =~ ^-[0-9alnpsw]+$ ]]; then
		words[last]+=e
	else
		words+=(-e)
	fi
	printf '%s\n' "${words[@]}"
}

escape_double_quoted() {
	local text=${1//\\/\\\\}
	printf '%s' "${text//\"/\\\"}"
}

escape_c_string() {
	local text
	text=$(escape_double_quoted "$1")
	printf '%s' "${text//$'\n'/\\n}"
}

escape_shell_single_quoted() {
	printf '%s' "${1//\'/\'\\\'\'}"
}

shell_command() {
	local interpreter=$1
	shift
	local command=$interpreter argument
	for argument in "$@"; do
		command+=" $argument"
	done
	printf "%s'%s'" "$command" "$(escape_shell_single_quoted "$core_body")"
}

emit() {
	local extension=$1 text=$2
	candidate_count=$((candidate_count + 1))
	printf '%s' "$text" >"$out_dir/$candidate_count.$extension"
}
