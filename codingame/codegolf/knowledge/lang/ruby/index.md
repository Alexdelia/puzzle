# Ruby

- [env](env.md): Ruby 3.1.2 (`_1`, endless def, `{x:}` shorthand; no `it`), bignums, `prime`/`set` requirable, verified
- gsub per row: `t.gsub(/./){r[$&.sum%32*l-l,l]}` after `t.tr("^a-zA-Z\n",?[)` (ascii-art ruby 90, verified locally)
- [exec-perl](exec-perl.md): `exec"perl -anE'..'"` wrapper beats native for interactive line puzzles, local only
- [shebang-p-chain](shebang-p-chain.md): `#!ruby -lp` and chain Kernel gsub/gsub!/chop!/tr! on $_
- [pop-pair-sum](pop-pair-sum.md): `y.sort!.sum{y.pop-_1}` pair-end sum of sorted array, verified
- [mayan-tokens](mayan-tokens.md): token read, default-arg recursive lambda, send operator, digits(20) printer
- [slice-sign-letter](slice-sign-letter.md): `" SN"[v,v*v]` sign->S/N/"", `$.` is 2 on first line of -alp file
- [prefix-hash-gsub](prefix-hash-gsub.md): `_1.gsub(/./){h[$`+$&]=1}` all prefixes of a line, 51 bytes total
- [map-growing-list](map-growing-list.md): `k.map{}` + `<<` chain flood work list, verified surface
- the-fall-episode-3 native port of the python solver is 791 B; interactive Ruby needs `$>.sync=1` (piped stdout buffers, timeouts), `x ?a:b` with a constant `W:` parses as label
- [rle-gsub-rows](rle-gsub-rows.md): gsub RLE decode, `w=gets.to_i`+`h=s.size/w`, `$q` nil-default flag (music-scores ruby 191)
