escape_php_single_quoted() {
	local text=$1 escaped= index char next
	for ((index = 0; index < ${#text}; index++)); do
		char=${text:index:1}
		next=${text:index+1:1}
		case $char in
		\') escaped+="\\'" ;;
		\\) [[ -z $next || $next == [\\\'] ]] && escaped+='\\' || escaped+='\' ;;
		*) escaped+=$char ;;
		esac
	done
	printf '%s' "$escaped"
}

escape_php_backtick() {
	local text=$1 escaped= index char next
	for ((index = 0; index < ${#text}; index++)); do
		char=${text:index:1}
		next=${text:index+1:1}
		case $char in
		\`) escaped+='\`' ;;
		\$) [[ $next == [A-Za-z_{] || $next == [![:ascii:]] ]] && escaped+='\$' || escaped+='$' ;;
		\\) [[ -z $next || $next == [nrtvef\\\$\"\`0-7xu] ]] && escaped+='\\' || escaped+='\' ;;
		*) escaped+=$char ;;
		esac
	done
	printf '%s' "$escaped"
}

shell_heredoc_command() {
	local command=$1 argument
	shift
	for argument in "$@"; do
		command+=" $argument"
	done
	[[ $command == "bash -c" ]] && command+=" "
	printf '%s"$(cat<<\\E\n%s\nE\n)"' "$command" "$core_body"
}

emit_php() {
	local command=$1
	emit php "<?system('$(escape_php_single_quoted "$command")');"
	[[ $'\n'$command$'\n' == *$'\n'P$'\n'* ]] || emit php "<?system(<<<'P'"$'\n'"$command"$'\n'"P);"
	[[ $kind == io ]] && emit php "<?=\`$(escape_php_backtick "$command")\`;"
	return 0
}

emit_php_core() {
	local interpreter=$1
	shift
	emit_php "$(shell_command "$interpreter" "$@")"
	[[ $'\n'$core_body$'\n' == *$'\n'E$'\n'* ]] || emit_php "$(shell_heredoc_command "$interpreter" "$@")"
}
