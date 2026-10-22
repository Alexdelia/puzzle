String.(let rec m t u i=try sub u i(length t)=t||m t u(i+1)with _->false
and g s t j=let u=(if m s t 0 then""else s)^sub t(length t-j)j in if m t u 0 then u else g s t(j+1)
and f s=try f(g s(read_line())0)with _->length s in print_int(f(read_line();"")))
