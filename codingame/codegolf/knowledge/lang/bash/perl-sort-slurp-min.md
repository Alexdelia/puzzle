# perl -0aE sort min

`perl -0aE'shift@F;say 0+(sort{$a*$a-$b*$b||$b-$a}@F)[0]'` (56, 17/17): -0a slurps all tokens into @F, `shift@F` drops N, `0+(list)[0]` gives 0 on empty list (N=0). Beats the pure bash loop (62: `read;read c a;for t in $a;{ c=$[t*t<c*c+(t>c)?t:c];};echo $[c]`).

- `abs$a-abs$b` parses as `abs($a-abs$b)`: wrong, use squares
- `say 0+(...)[0]` is 1 byte under `say+(...)[0]||0`; `-0aE` 1 under `-E'<>;..split$",<>'`
