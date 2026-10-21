# Clojure: .replaceAll beats clojure.string/replace

Verified chuck-norris 177: `(.replaceAll s "(0)+|1+" " 0$1 $0")` on a String (reflection fine), `$0` whole match, unmatched group empty. 12 bytes under `clojure.string/replace` + `#"..."`. `.trim` (9) beats `(subs x 1)` (10) to drop a leading space. Bits: `(subs(Integer/toString(+ 128 %)2)1)` over `(.getBytes(read-line))`.
