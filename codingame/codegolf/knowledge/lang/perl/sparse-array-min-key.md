# Sparse array finds argmin-by-key, `"@a"*1` numifies the winner

`@a` is a sparse array indexed by a numeric key `k(t)`; assign `$a[k($_)]=$_` for every token. After the loop, `"@a"` is a string of `$"`-joined elements where every undef hole is empty, so `"@a"*1` (numeric coercion of the leading number) is the value stored at the *smallest assigned index*: the argmin by `k`. No sort, no min loop.

Temperature perl 42 -> 40, CG 100% rank #21 (2026-10-24), passes tests + recovered hidden:

```perl
<>;map$G[$_**2-$_]=$_,glob<>;print"@G"*1
```

- key `$_**2-$_` = `t(t-1)` is monotonic in `|t|` (`-a` -> `a(a+1)`, `+a` -> `a(a-1)`) so it is the exact min-|t| with positive-wins-tie comparator of `_shared/abs-tie-comparator.md`, in 8 B
- alternatives: `abs` (3 B) as key is 35 B but last-wins at an `abs` tie, so it loses when the negative comes last (temperature V6 `... -7 ...`); `map$G[abs]=$_,sort glob<>;` restores positive-wins because all `-x` strings sort before all `x` (costs the 5 saved), same 40 B
- empty input: `"@G"` is `""`, `*1` gives `0`, so N=0 needs no `+0` (the 40 B program ends `print"@G"*1`, `*1`/`/1`/`-0` are the same length; `|0`/`^0` coerce negative values to unsigned 64-bit, do not use)
- `<>;` (3 B) must consume line 1 first: `glob<>` only reads one line
- value cell is `=$_` 3 B; `mapKEY=VALUE,LIST` with `,` before `glob<>` is 1 B under `for ... glob<>`
- a C-shift variant `$G[3e3-$_**2+$_]=$_ ...;print pop@G` (answer at the largest index) needs `print pop@G` (11 B) and a constant >= max key, so it is never shorter; `$-` is not abs (`$-=-5` is 0)
