# Byte-table lookup and global init (vox-codei-episode-2, verified locally node 20.17)

- `Buffer.from("ABC").indexOf(code)` takes a number: finds a byte value in an ASCII table, ~10 B shorter than `"ABC".indexOf(String.fromCharCode(code))`. Only for ASCII (UTF-8 multi-byte chars break byte indexing)
- `[w,h,T]=readline().split` `` : destructuring a short array still declares the missing element as an implicit global (`T` undefined), so `t==T` works with no `var`
- `for(i in s)` over a string gives index strings, `i*i` coerces: `for(i in s)k+=(s[i]>"?")*i*i` is the shortest position-weighted hash of '@' cells ('@' is the only char above '?')
- readline at EOF returns null forever: an endless `for(;;)` loop spins after the game ends; the referee stops it, harmless
