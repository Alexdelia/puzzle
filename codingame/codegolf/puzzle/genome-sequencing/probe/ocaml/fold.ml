open String
let rec m t u i=try sub u i(length t)=t||m t u(i+1)with _->false
let rec g s t k=let u=s^sub t k(length t-k)in if m t u 0 then u else g s t(k-1)
let rec f s=try let t=read_line()in f(g(if m s t 0 then""else s)t(length t))with _->length s;;print_int(f(read_line();""))
