rust_system_prefix='unsafe extern{safe fn system(_:&str);}fn main(){system('

rust_raw_hashes() {
	local text=$1 hashes=
	while [[ $text == *"\"$hashes"* ]]; do
		hashes+='#'
	done
	printf '%s' "$hashes"
}

rust_literals() {
	local text=$1 hashes
	printf '%s\0' "\"$(escape_double_quoted "$text")\""
	if [[ $text == *[\\\"]* ]]; then
		hashes=$(rust_raw_hashes "$text")
		printf '%s\0' "r$hashes\"$text\"$hashes"
	fi
}

rust_arguments() {
	local arguments= argument
	for argument in "$@"; do
		arguments+="\"$(escape_double_quoted "$argument")\","
	done
	printf '%s' "$arguments"
}

emit_rust_nul() {
	candidate_count=$((candidate_count + 1))
	printf '%s\0%s' "$1" "$2" >"$out_dir/$candidate_count.rs"
}

emit_rust_command() {
	local interpreter=$1 body=${!#} literal arguments
	shift
	arguments=$(rust_arguments "${@:1:$#-1}")
	[[ $body == *$'\r'* ]] && return 0
	while IFS= read -r -d '' literal; do
		emit rs "fn main(){std::process::Command::new(\"$interpreter\").args([$arguments$literal]).status();}"
		[[ $kind == interactive ]] || emit rs "fn main(){std::process::Command::new(\"$interpreter\").args([$arguments$literal]).spawn();}"
	done < <(rust_literals "$body")
}

emit_rust_system() {
	local command=$1 hashes
	[[ $command == *$'\r'* ]] && return 0
	emit rs "$rust_system_prefix\"$(escape_double_quoted "$command")\\0\");}"
	emit_rust_nul "$rust_system_prefix\"$(escape_double_quoted "$command")" '");}'
	if [[ $command == *[\\\"]* ]]; then
		hashes=$(rust_raw_hashes "$command")
		emit_rust_nul "${rust_system_prefix}r$hashes\"$command" "\"$hashes);}"
	fi
}

emit_rust() {
	local interpreter=$1
	shift
	emit_rust_command "$interpreter" "$@"
	emit_rust_system "$(shell_command "$interpreter" "${@:1:$#-1}")"
}
