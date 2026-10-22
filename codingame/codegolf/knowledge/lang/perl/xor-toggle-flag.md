# Perl: xor toggle flag in a regex (verified locally + CG, blunder-episode-1 perl 269)

- `$b^=/B/` toggles a flag in 7 bytes (vs `$b=!$b if/B/` 12): values undef, 1, 0, 1, 0... (`/B/` fails as dualvar ""/0, so `^` stays numeric)
- problem: 0 and undef differ when interpolated (`X$b` = `X0` vs `X`). Fix: append the constant to the subject, `"$c0"`-style: `"$m[$i]0"!~/#|X$b/` treats undef and 0 alike ("X0" matches `X` and `X0`, not `X1`); +3 bytes, net -3
- PITFALL: `$a.0!~/re/` parses as `$a.(0!~/re/)` (`.` looser than `=~`/`!~`), use the quoted form
- PITFALL: `X.{$b}` with undef b is a literal `{}`, not `{0}`
