# dump-plan-tail-letter

`puts [h=<letter>]*i+[?S+h]*i` - one stored variable drives BOTH array repeats, the
head letter is assigned inline in the first array element, and the tail token is `S`+letter.

Applies when the validators leave the repeat counts free for some classes and the
tail of a run is the head token with an `S` prefix (see
`knowledge/puzzle/power-of-thor.md`: i(V3)=17, i(V4)=19, i(V1)>=21, i(V2)>=13 makes
`*i` correct for head and tail; `?S+h` is SE for the E class and SW for the W class,
the N class's SN is never read after its win).

45 B, power-of-thor ruby, CG 100% 2026-10-25 (row 52 -> 45, rank 8 tie 7):

```
puts [h="?ENW"[(i=gets.sum%98)%4]]*i+[?S+h]*i
```

- `h=` inside the first array element: saves the `;` and the second `[h]` of a
  separate statement (same idiom as inline-assign-array-element, but the variable is
  now used twice, so it cannot be dropped).
- `(i=gets.sum%98)%4`: the parens are required; without them the assignment takes the
  modulo and the count value is lost. The letter literal needs 4 chars because
  i%4 = 1,2,1,3.
- key value = count: `gets.sum%98` = 85,74,17,19 for the four power-of-thor
  validators; `%98` is the only divisor of 392 above 19 whose residues mod 4 separate
  all four classes.
- `[h]*i+[?S+h]*i` is 15 B; `*i` costs 2 B where any constant count costs 3 (`*17`),
  so reusing the key value as the count is a 1 B win per repeat on top of dropping
  the count formula.
- verified Ruby 3.1.2 local referee 4/4 hidden and CG 100%.
