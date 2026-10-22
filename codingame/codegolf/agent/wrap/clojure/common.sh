clojure_arguments() {
	local arguments= argument
	for argument in "$@"; do
		arguments+="\"$(escape_double_quoted "$argument")\""
	done
	printf '%s' "$arguments"
}

clojure_wrapper() {
	local arguments
	arguments=$(clojure_arguments "$@")
	if [[ $kind == interactive ]]; then
		printf '(ns Player)(defn -main[_](..(ProcessBuilder.[%s])inheritIO start waitFor))' "$arguments"
	else
		printf '(ns Solution(:gen-class))(..(ProcessBuilder.[%s])inheritIO start)' "$arguments"
	fi
}

clojure_main_wrapper() {
	printf '(ns Solution(:gen-class))(defn -main[](..(ProcessBuilder.[%s])inheritIO start waitFor))' "$(clojure_arguments "$@")"
}

emit_clojure() {
	emit clj "$(clojure_wrapper "$@")"
	[[ $kind == interactive ]] || emit clj "$(clojure_main_wrapper "$@")"
}
