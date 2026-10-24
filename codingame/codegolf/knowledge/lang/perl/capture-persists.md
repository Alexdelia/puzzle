# capture-persists

`$1`, `$2`, ... keep their last successful capture across later regexes that have no capture groups and across failed matches (verified Perl 5.42; no reason to differ on CG 5.34). Use it to read one header field once in a `-p`/`-n` loop without an assignment:

```
#!perl -pl
/(\d+ ){3}/;$_=($1-$.)%6?WAIT:BLOCK
```

On the init line `/(\d+ ){3}/` sets `$1` to the third field (with trailing space, numifies fine); turn/elevator/NONE lines do not match it. `$1` survives a later `/L/` (no groups). Costs only the regex instead of `$s=$F[2]if$.<2` (and needs no `-a`). Negative: a later match with groups overwrites it.
