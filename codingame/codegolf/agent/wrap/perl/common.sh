perl_escape_quoted() {
	perl -e '$_ = $ARGV[0]; $d = quotemeta $ARGV[1]; s/(\\+)(?=$d|\z)|(\\\\+)/$1$1$2$2/g; s/$d/\\$&/g; print' -- "$1" "$2"
}

perl_quote_delimiter() {
	local delimiter
	for delimiter in '~' '!' '|' '^' '#' '%' '@' ';' ':' ','; do
		[[ $1 != *"$delimiter"* ]] && printf '%s' "$delimiter" && return
	done
	return 0
}

emit_perl_exec() {
	local prefix=$1 text=$2 delimiter quote=q
	emit pl "$prefix'$(perl_escape_quoted "$text" "'")'"
	[[ $prefix =~ [A-Za-z0-9_]$ ]] && quote=" q"
	delimiter=$(perl_quote_delimiter "$text")
	[[ -n $delimiter ]] && emit pl "$prefix$quote$delimiter$(perl_escape_quoted "$text" "$delimiter")$delimiter"
	[[ $text != *$'\n\n'* && $text != $'\n'* && $text != *$'\n' ]] && emit pl "$prefix<<''"$'\n'"$text"$'\n'
	return 0
}
