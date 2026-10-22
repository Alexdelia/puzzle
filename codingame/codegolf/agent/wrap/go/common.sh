go_raw_strings() {
	local argument
	for argument in "$@"; do
		printf ',`%s`' "$argument"
	done
}

go_quoted_strings() {
	local argument
	for argument in "$@"; do
		printf ',"%s"' "$(escape_c_string "$argument")"
	done
}

go_spliced_strings() {
	local argument
	for argument in "$@"; do
		printf ',`%s`' "${argument//'`'/'`+"`"+`'}"
	done
}

go_wrapper() {
	local environment
	for environment in nil 'Environ()'; do
		printf 'package main\nimport."syscall"\nfunc main(){Exec("%s",[]string{""%s},%s)}\0' "$1" "$2" "$environment"
	done
}

emit_go_wrappers() {
	local wrapper
	while IFS= read -r -d '' wrapper; do
		emit go "$wrapper"
	done < <(go_wrapper "$@")
}

emit_go() {
	local path=$1
	shift
	if [[ $* == *'`'* ]]; then
		emit_go_wrappers "$path" "$(go_spliced_strings "$@")"
	else
		emit_go_wrappers "$path" "$(go_raw_strings "$@")"
	fi
	emit_go_wrappers "$path" "$(go_quoted_strings "$@")"
}
