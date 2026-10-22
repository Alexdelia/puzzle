groovy_single_quoted() {
	local text=${1//\\/\\\\}
	text=${text//\'/\\\'}
	printf "'%s'" "${text//$'\n'/\\n}"
}

groovy_triple_quoted() {
	local text=${1//\\/\\\\}
	printf "'''%s'''" "${text//\'/\\\'}"
}

groovy_slashy() {
	[[ -n $1 && $1 != '*'* && $1 != *\\ && $1 != *\\u* && ! $1 =~ \$[A-Za-z_{] ]] || return 1
	printf '/%s/' "${1//\//\\/}"
}

groovy_wrapper() {
	local literal=$1 arguments= argument
	shift
	for argument in "$@"; do
		arguments+="'$argument',"
	done
	local call="new ProcessBuilder($arguments$literal).inheritIO().start()"
	[[ $kind == interactive ]] && call+='.waitFor()'
	printf '%s' "$call"
}

emit_groovy() {
	local single literal
	single=$(groovy_single_quoted "$core_body")
	emit groovy "$(groovy_wrapper "$single" "$@")"
	for literal in "$(groovy_triple_quoted "$core_body")" "$(groovy_slashy "$core_body")"; do
		[[ -n $literal ]] && ((${#literal} < ${#single})) && emit groovy "$(groovy_wrapper "$literal" "$@")"
	done
	return 0
}
