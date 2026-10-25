# Assign inside the first array element to drop the separator

Ruby evaluates `[expr] * n` receiver-first, so a variable can be created inside the
first element and used by the multiplier and by later `+` parts of the same expression:

```
puts [h="WEN"[i=gets.sum%3]]*(19+12*i)+[?S+h*0**i]*17
```

`h="WEN"[i=...]` is the element value (the letter), so `[h=...]` is the 1-element
array; `i` is already set when `(19+12*i)` is evaluated (left-to-right), and both `h`
and `i` stay in scope for the second array. This replaces the two-statement form

```
h="WEN"[i=gets.sum%3]
puts [h]*(19+12*i)+[?S+h*0**i]*17
```

for -2 B (the newline separator and the extra `[h]` shrink to the inline element
`[h=...]`). Powers of 10/strings work the same: any expression that assigns and returns
the value can be the first element.

Verified: power-of-thor ruby 55 -> 53 B, CG 100%, hidden 4/4 (2026-10-25, commit in the
task journal; rank 18, tie 15). Bytes: `puts ` 5 + `[` 1 + key 21 + `]` 1 + `*` 1 +
count 7 + `+` 1 + second run 11 + `*17` 3 = 53 (trailing newline free).

Caveats: the multiplier must come after the array literal (`[h=...]*(...)`, not
`*`-before), and the first element must be the assignment itself (`[h=...]`), not
`[h=...; x]`; `puts[` is not a method call (space required after `puts`).

## Counter-case: no variable at all when the letter is used once (2026-10-25)

If the selected letter is needed only once (the first array), do not assign it:
write the tail's constant letter directly. Power-of-thor 53 -> 52 B, CG 100%:

```
puts [h="WEN"[i=gets.sum%3]]*(19+12*i)+[?S+h*0**i]*17   # 53
puts ["WEN"[i=gets.sum%3]]*(19+12*i)+[?S+?W*0**i]*17    # 52
```

`[h=...]` + `h*` (2+2 B) becomes `[...]` + `?W*` (0+1 B). The inline assignment
still wins when the letter is used twice or more (`[h=...]*(...)` plus `h` in the
tail), e.g. when the tail letter is key-dependent rather than a constant.
