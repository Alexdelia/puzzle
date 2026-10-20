# sprintf vector flag `%v`

- `sprintf"%v07b",$c` formats `ord` of each char of `$c` (flags and width after the `v`): for one char, 2 bytes under `sprintf"%07b",ord$c`, verified (chuck-norris, 14/14 on referee)
- `%07vb` (width before `v`) is not parsed: prints the literal format
- on a longer string, the results are joined with `.`; `%*v07b` takes the join string as an extra first argument (`"%*v07b","",$_`), costing +3 over the single-char form inside `s/./.../ge`: not shorter for per-char loops, unverified gain elsewhere
