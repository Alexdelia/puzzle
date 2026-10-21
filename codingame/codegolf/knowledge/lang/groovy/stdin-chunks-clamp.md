# Groovy stdin destructure, regex chunks, list indexing

Verified on ascii-art (local referee 13/13, CG 116 rank 3, 2026-10-21):

- `(l,h,t)=a=System.in.readLines()`: multi-assign from the list, whole list kept in `a`; `l` stays a String and still works as `/.{$l}/` (slashy GString interpolation), so no `as int`
- `(r=~/.{$l}/)[i]`: Matcher `getAt`: `-1` = last match, index past the end or below `-size` throws. Clamp the index: `[x,27].min()-1`
- `List.getAt(int)`: below `-size` throws (`ArrayIndexOutOfBounds`), past the end returns null; `?:` fallback works for nulls only
- `byte[]` has no `sum(init){}`/`sum{}`; `collect{}.sum()` on a List of Strings concatenates (first element is the start, no `0+` problem) and is 1 byte shorter than `.join()`
- `for(r in a[3..-1])println expr` (no braces): the `expr` can assign a variable on first use inside the closure
- `y=c&95^64` (letters 1..26, `@` and backtick 0, others >=27 or 0): `[y,27].min()-1` is the glyph index with `-1` = last glyph
