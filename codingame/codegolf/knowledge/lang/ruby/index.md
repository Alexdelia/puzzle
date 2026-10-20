# Ruby

- [env](env.md): Ruby 3.1.2 (`_1`, endless def, `{x:}` shorthand; no `it`), bignums, `prime`/`set` requirable, verified
- gsub per row: `t.gsub(/./){r[$&.sum%32*l-l,l]}` after `t.tr("^a-zA-Z\n",?[)` (ascii-art ruby 90, verified locally)
- [exec-perl](exec-perl.md): `exec"perl -anE'..'"` wrapper beats native for interactive line puzzles, local only
- [shebang-p-chain](shebang-p-chain.md): `#!ruby -lp` and chain Kernel gsub/gsub!/chop!/tr! on $_
- [pop-pair-sum](pop-pair-sum.md): `y.sort!.sum{y.pop-_1}` pair-end sum of sorted array, verified
