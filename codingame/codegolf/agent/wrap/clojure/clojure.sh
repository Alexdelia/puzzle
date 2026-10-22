clojure_wrapper() {
	local arguments= argument
	for argument in "$@"; do
		arguments+="\"$(escape_double_quoted "$argument")\""
	done
	if [[ $kind == interactive ]]; then
		printf '(ns Player)(defn -main[_](..(ProcessBuilder.[%s])inheritIO start waitFor))' "$arguments"
	else
		printf '(ns Solution(:gen-class))(..(ProcessBuilder.[%s])inheritIO start)' "$arguments"
	fi
}
