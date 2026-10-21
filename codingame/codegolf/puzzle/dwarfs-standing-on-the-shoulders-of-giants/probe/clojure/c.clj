(ns Solution(:gen-class))(defn -main[](println(+ 2(#(count(re-seq #"(?s)(\d+) (?=.*

.* \1
)"(str % "
"%)))(slurp *in*)))))