ruby_delimited() {
	perl -0777 -e '
		my ($delimiter, $text) = @ARGV;
		$text =~ s{(\\*)(\Q$delimiter\E|\z)|(\\+)}{
			defined $3 ? "\\" x (2 * length($3) - 1) : "\\" x (2 * length($1)) . (length($2) ? "\\$2" : "")
		}ge;
		print $text;
	' "$1" "$2"
}

ruby_double_quoted() {
	local text
	text=$(escape_double_quoted "$1")
	text=${text//'#{'/'\#{'}
	text=${text//'#$'/'\#$'}
	printf '%s' "${text//'#@'/'\#@'}"
}

ruby_free_delimiter() {
	local delimiter
	for delimiter in '|' '!' '~' '^' '`' ':' ';' ',' '.' '/' '?' '-' '_' '+' '*' '&' '=' '@' '%' '$'; do
		[[ $1 != *"$delimiter"* ]] && printf '%s' "$delimiter" && return
	done
	printf '|'
}

ruby_heredoc_terminator() {
	local terminator
	for terminator in E F G H I J K L M N; do
		grep -qx -- "$terminator" <<<"$1" || { printf '%s' "$terminator" && return; }
	done
	printf 'EOF_RUBY_WRAP'
}

ruby_exec() {
	local arguments=$1 text=$2 delimiter terminator
	delimiter=$(ruby_free_delimiter "$text")
	terminator=$(ruby_heredoc_terminator "$text")
	emit rb "exec$arguments\"$(ruby_double_quoted "$text")\""
	emit rb "exec$arguments'$(ruby_delimited "'" "$text")'"
	if [[ -z $arguments ]]; then
		emit rb "exec %q$delimiter$(ruby_delimited "$delimiter" "$text")$delimiter"
	else
		emit rb "exec$arguments%q$delimiter$(ruby_delimited "$delimiter" "$text")$delimiter"
	fi
	if [[ -z $arguments ]]; then
		emit rb "exec <<'$terminator'"$'\n'"$text"$'\n'"$terminator"
	else
		emit rb "exec$arguments<<'$terminator'"$'\n'"$text"$'\n'"$terminator"
	fi
}

ruby_exec_interpreter() {
	local arguments= argument
	for argument in "${@:1:$#-1}"; do
		arguments+="\"$(ruby_double_quoted "$argument")\","
	done
	ruby_exec "$arguments" "${!#}"
	ruby_exec "" "$(shell_command "${@:1:$#-1}")"
}
