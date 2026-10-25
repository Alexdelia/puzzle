# Statement-modifier `while<>` instead of a `-nl` shebang

A Perl one-liner-style solution that uses `#!perl -nl` and prints explicitly can often drop the
shebang and end the body with `while<>`:

```
# 56 B (shebang + explicit print)
#!perl -nl
/ /;$|=print"S"x(...).(W,E,N)[$`%3]

# 55 B (no shebang; same output)
/ /,$|=print"S"x(...).(W,E,N)[$`%3],$/while<>
```

Verified on CG 2026-10-25, power-of-thor perl 56 -> 55 (then 54 with a cheaper phase), 100%.

- `EXPR while<>` (2 words) is a statement modifier: `<>` in scalar context reads one line per
  iteration into `$_` (same as `-n`), `$.` counts lines from 1, loop ends at EOF.
- Without `-l` the print has no newline: add `,$/` to the print list (3 B). The shebang line
  `#!perl -nl` costs 11 B including its newline; `while<>` + `,$/` costs 10 B, so the net saving
  is 1 B.
- `-l` also chomps `$_`; not needed when the body only numifies the line or matches a pattern
  without `$` in the pattern. `/ /` still matches the init line; `$`` survives the later failed
  matches in plain code exactly as under `-n` (`-p` resets it).
- The body must be one expression statement, or statements joined by the comma operator, because
  the modifier binds the whole statement: `/ /,$|=print"...",$/while<>` runs the match then the
  print each iteration.
- Multi-line output: this trick gives one newline per print, so it does not help plan-dump code
  that embeds its own newlines; it is for per-line interactive output.
- Trap: `for<>` is NOT a shorter (5 B) substitute for `while<>`. A statement modifier `for LIST`
  evaluates the list in list context, so `<>` slurps the whole input before the first iteration.
  On interactive stdin this blocks forever (power-of-thor, 52 B candidate, local hidden 0/4 with
  "no answer within 1000 ms" on every case, 2026-10-25). Only scalar-context `while<>` reads one
  line per iteration; there is no 5 B loop form.
