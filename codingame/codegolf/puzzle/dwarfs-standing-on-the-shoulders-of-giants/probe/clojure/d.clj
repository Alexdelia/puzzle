(ns Solution(:gen-class))(defn -main[](prn(+ 2(#(count(re-seq #"(?s)(\d+) (?=.*

.* \1
)"(str % "
"%)))(slurp *in*)))))