# Skip the `s=''` init: assign in a ternary whose else-branch is the first-line case

Verified (genome-sequencing, CG 100%, 2026-10-21): in `System.in.eachLine{t->s=cond?...:''}` the script-binding var `s` is never read on the first line when the first line (digit count N) takes the `:''` branch, so no separate `s=''` is needed (saves 5 bytes).

- `t>'9'` as "is a letter line" (digit lines compare lower), nested ternary `a?b?x:y:z` needs no parens
- `=~` returns a Matcher, truthy when found: `t=~s?` is substring test (s as regex, fine for plain letters), parenthesise the left side when it holds `+`: `(s+t)=~u`
- `String.drop(n)` is safe for any n (unlike `t[n..-1]`)
- `System.in.eachLine{t->` (22) beats `System.in.text.split().tail().each{t->` (38) once the N line is guarded
