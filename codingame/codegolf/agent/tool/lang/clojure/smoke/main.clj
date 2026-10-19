(ns Solution
  (:gen-class))

(defn -main [& args]
  (let [[a b] (map #(Long/parseLong %) (.split (read-line) " "))]
    (println (+ a b))))
