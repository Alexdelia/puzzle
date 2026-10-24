# Read all remaining lines twice: `(<>)x2`

`@q=map$q+=$_,(<>)x2` reads every remaining line once (list-context `<>`), repeats the list (`(LIST)x2`) and builds the prefix sums of the doubled queue: no `@p`, no `-0ap` shebang. Lines keep their `\n`, numeric use ignores it.

With `($l,$c)=glob<>` for the header and `$_` as accumulator plus a final `;print`, no-shebang input costs 35 bytes against `#!perl -0ap` + `($l,$c,$n,@p,$_)=@F;@q=map$q+=$_,@p,@p;` = 52 (both forms leave `$_` usable as the state): -10 net after the `;print`.

`<>x2` without parens is scalar repetition (one line): parens needed.

Verified: roller-coaster perl 105, local 16/16, CG-accepted 2026-10-22 (submit.sh).

## Addendum 2026-10-24: repeating the *prefix list* is not the doubled prefix

`(map$q+=$_,<>)x2` maps once and then repeats the resulting *values* (`P1..PN,P1..PN`), it does not extend the cumulative sum. It is wrong whenever the search needs the second lap (`S+Pj`). `@q=map$q+=$_,(<>)x2` is the correct doubled prefix (and leaves `$q`=2S).

If a scalar equal to `S` is wanted, `@q=map$q+=$_,<>;push@q,@q` gives the correct doubled list with `$q`=S: the 6 bytes saved on `S` (`$_%$q` instead of `$_%($q/2)`, `$q>=` instead of `$q/2>=`) exactly pay for the 11-byte `push@q,@q` (measured 105 both ways). Verified roller-coaster perl 2026-10-24 (scratch v3/v4, CG perl 5.34).
