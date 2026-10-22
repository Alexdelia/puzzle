quote_single() {
	local quoted="'$(escape_shell_single_quoted "$1")'"
	while [[ $quoted == *"\\'''\\'"* ]]; do
		quoted=${quoted//"\\'''\\'"/"\\'\\'"}
	done
	[[ $quoted == "''\\'"* ]] && quoted=${quoted#"''"}
	[[ $quoted == *"\\'''" ]] && quoted=${quoted%"''"}
	printf '%s' "$quoted"
}

quote_double() {
	local text=$1 quoted=\" index char next
	for ((index = 0; index < ${#text}; index++)); do
		char=${text:index:1}
		next=${text:index+1:1}
		case $char in
		[\"\`]) quoted+=\\ ;;
		\\)
			case $next in
			'' | [\$\`\"\\] | $'\n') quoted+=\\ ;;
			esac
			;;
		\$)
			case $next in
			'' | [' )};,.:/+=<>%^&|~"`\'] | \\ | $'\n') ;;
			*) quoted+=\\ ;;
			esac
			;;
		esac
		quoted+=$char
	done
	printf '%s"' "$quoted"
}

quote_ansi() {
	local text=${1//\\/\\\\}
	printf "\$'%s'" "${text//\'/\\\'}"
}

round_trips() {
	(
		set +u
		eval "decoded=$1" 2>/dev/null && [[ $decoded == "$2" ]]
	)
}

emit_quoted() {
	local prefix=$1 text=$2 quote quoted
	for quote in quote_single quote_double quote_ansi; do
		quoted=$($quote "$text")
		round_trips "$quoted" "$text" && emit sh "$prefix$quoted"
	done
}

heredoc_delimiter() {
	local delimiter
	for delimiter in E e X Z Q _; do
		[[ $'\n'$1$'\n' == *$'\n'$delimiter$'\n'* ]] || {
			printf '%s' "$delimiter"
			return
		}
	done
	return 1
}

emit_heredoc() {
	local command=$1 text=$2 delimiter
	delimiter=$(heredoc_delimiter "$text") || return 0
	emit sh "$command /dev/fd/3 3<<\\$delimiter"$'\n'"$text"
}
