(ns Solution(:gen-class))(defn -main[](prn(count(reduce #(cond(.contains %2 %)%2(.contains % %2)% 1(.replaceFirst(str % "
"%2)"(.*)
\\1""$1"))"" (re-seq #"[A-Z]+"(slurp *in*))))))