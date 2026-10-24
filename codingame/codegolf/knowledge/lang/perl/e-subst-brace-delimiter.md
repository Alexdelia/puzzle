# Braced `s{}{}ge` lets inner `//` parse for free

With an outer substitution written `s/PAT/CODE/ge`, an unescaped inner `//` in CODE
(`s/O//`, `split//`, `y/O//`, a literal `//`) ends the replacement early:

    s/x/$u=~s/O//&&print"m"/e   # Unknown regexp modifier "/O", syntax error

The fix costs nothing when PAT has no brace: `s{PAT}{CODE}ge` is the same byte count
as `s/PAT/CODE/ge` (`s{` + `}` replace `s/` + `/`), and inner `//` parse normally.
Escaping the inner slashes costs 1 B each; `split'',` costs 1 B over `split//`.

Verified locally on CG perl 5.34 (`s{x}{...}e`, `s{\d+ }{...}ge`) and on the
surface referee (variant with `@u=split//;$u[$_]=~s/O//` only compiles with braces).
Trap: PAT must not contain unbalanced `{}` (use another free delimiter or escape).
