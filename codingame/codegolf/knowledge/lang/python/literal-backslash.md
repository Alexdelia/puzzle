# Python string literals: one backslash for Perl regex escapes

CG Python 3.11.5 keeps an unknown escape sequence as a literal backslash (only an
invalid-escape warning to stderr; stdout is untouched). So an embedded Perl/sh
regex class needs no doubling:

    os.system("perl -lnE'.../(\\w)|./g...'")   # old, safe everywhere
    os.system("perl -lnE'.../(\w)|./g...'")    # -1 B, CG 3.11.5

Same for `\pL`, `\d`, `\b`, `\S` in any wrapper core.

Verified: ascii-art python 95 -> 93 B, CG 100 % (submission 2026-10-25, rank #4 tie #3,
`\\pL` -> `\w` also using the shorter class). Local 7/7 hidden + 6/6 tests.

Caveats:

- Python 3.12+ promotes it to `SyntaxWarning`, later versions may error; CG is pinned
  3.11.5 (`agent/state/lang/python.json`), re-check after a CG version bump
- the stderr warning is harmless for stdout-compared io puzzles
- only saves the doubling, not a real escape: `\n`, `\t`, `\\` must stay escaped
