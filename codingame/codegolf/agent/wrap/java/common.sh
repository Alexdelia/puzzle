java_quoted() {
	local text
	text=$(escape_c_string "$1")
	printf '"%s"' "${text//$'\r'/\\r}"
}

java_text_block() {
	[[ $1 =~ (^|$'\n')[^[:space:]] ]] || return 0
	printf '"""\n%s"""' "$(perl -0777 -pe 's/\\/\\\\/g;s/"(?=""|\z)/\\"/g;s/\r/\\r/g;s/[ \t](?=\n|\z)/$&eq" "?"\\s":"\\t"/ge' < <(printf '%s' "$1"))"
}

java_wrapper() {
	local literal=$1 wait=$2 arguments= argument
	shift 2
	for argument in "$@"; do
		arguments+="$(java_quoted "$argument"),"
	done
	local main=Solution
	[[ $kind == interactive ]] && main=Player
	printf 'interface %s{static void main(String[]a)throws Exception{new ProcessBuilder(%s%s).inheritIO().start()%s;}}' "$main" "$arguments" "$literal" "$wait"
}

emit_java() {
	local literal wait
	for literal in "$(java_quoted "$core_body")" "$(java_text_block "$core_body")"; do
		[[ -n $literal ]] || continue
		for wait in .waitFor\(\) ''; do
			[[ -z $wait && $kind == interactive ]] && continue
			emit java "$(java_wrapper "$literal" "$wait" "$@")"
		done
	done
	return 0
}
