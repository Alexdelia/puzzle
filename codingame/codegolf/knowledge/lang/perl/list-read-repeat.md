# Read all remaining lines twice: `(<>)x2`

`@q=map$q+=$_,(<>)x2` reads every remaining line once (list-context `<>`), repeats the list (`(LIST)x2`) and builds the prefix sums of the doubled queue: no `@p`, no `-0ap` shebang. Lines keep their `\n`, numeric use ignores it.

With `($l,$c)=glob<>` for the header and `$_` as accumulator plus a final `;print`, no-shebang input costs 35 bytes against `#!perl -0ap` + `($l,$c,$n,@p,$_)=@F;@q=map$q+=$_,@p,@p;` = 52 (both forms leave `$_` usable as the state): -10 net after the `;print`.

`<>x2` without parens is scalar repetition (one line): parens needed.

Verified: roller-coaster perl 105, local 16/16, CG-accepted 2026-10-22 (submit.sh).
