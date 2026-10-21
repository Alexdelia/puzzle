(ns Solution(:gen-class))(defn -main[](let[s(slurp *in*)](println(+ 2(count(filter #(.contains s(str" "%"
"))(re-seq #"\d+(?= )"s)))))))