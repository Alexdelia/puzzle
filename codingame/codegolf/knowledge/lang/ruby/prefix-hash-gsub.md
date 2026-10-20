# Distinct prefixes via gsub into a hash (telephone-numbers ruby 51, local 15/15)

`gets;h={};$<.map{_1.gsub(/./){h[$`+$&]=1}};p h.size`: `$`+$&` is the prefix through the matched char, `.` skips `\n` so no empty key; `gets` skips the N line. Slices + `uniq` form is 59. CG submit not done (cap).
