scala_main() {
	local object=Solution
	[[ $kind == interactive ]] && object=Player
	printf 'object %s extends App{%s}' "$object" "$1"
}

scala_string() {
	local text
	text=$(escape_c_string "$1")
	printf '"%s"' "${text//$'\r'/\\r}"
}

scala_raw_string() {
	[[ $1 != *'"""'* && $1 != *'\u'* && $1 != *'"' ]] || return 1
	printf '"""%s"""' "$1"
}

scala_literals() {
	scala_string "$1"
	printf '\0'
	scala_raw_string "$1" && printf '\0'
	return 0
}

scala_shortest_literal() {
	local plain raw
	plain=$(scala_string "$1")
	raw=$(scala_raw_string "$1") && ((${#raw} < ${#plain})) && plain=$raw
	printf '%s' "$plain"
}

tokenizer_word() {
	local text=$1 word= quote= index char run next
	for ((index = 0; index < ${#text}; index++)); do
		char=${text:index:1}
		if [[ -n $quote ]]; then
			if [[ $char != [[:space:]\'\"\\] ]]; then
				run=${text:index}
				run=${run%%[[:space:]\'\"\\]*}
				next=${text:index+${#run}:1}
				[[ $next == [[:space:]] || $next == [\'\"] && $next != "$quote" ]] || {
					word+=$quote
					quote=
				}
				word+=$run
				index=$((index + ${#run} - 1))
				continue
			fi
			if [[ $char == "$quote" || $char == \\ ]]; then
				word+=$quote
				quote=
			fi
		fi
		if [[ -z $quote && $char == [[:space:]\'\"] ]]; then
			quote=\'
			[[ $char == \' ]] && quote=\"
			word+=$quote
		fi
		word+=$char
	done
	printf '%s' "$word$quote"
}

emit_process() {
	local command=$1 argument literal
	shift
	for argument in "$@"; do
		command+=" $(tokenizer_word "$argument")"
	done
	while IFS= read -r -d '' literal; do
		emit scala "$(scala_main "sys.process.Process($literal).!<")"
	done < <(scala_literals "$command")
}

emit_seq() {
	local literals= argument
	for argument in "$@"; do
		literals+=,$(scala_shortest_literal "$argument")
	done
	emit scala "$(scala_main "import sys.process._;Seq(${literals#,}).!<")"
}

emit_interpreter() {
	local interpreter=$1
	shift
	local flags=("$@")
	emit_process "$interpreter" "${flags[@]}" "$core_body"
	if [[ $interpreter != sh && $interpreter != bash ]]; then
		local last=$((${#flags[@]} - 1))
		emit_process "$interpreter" "${flags[@]:0:last}" "${flags[last]}$core_body"
	fi
	emit_seq "$interpreter" "${flags[@]}" "$core_body"
}
