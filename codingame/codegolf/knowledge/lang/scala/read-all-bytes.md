# System.in.readAllBytes: whole stdin as a mutable array, numeric chars

Verified (blunder-episode-1, local with the stub CG_SCALA and CG test game 7 + submit 100%, 2026-10-21):

- `val m=System.in.readAllBytes` is Array[Byte], 21 chars vs `scala.io.Source.stdin.toArray` 36 (`io.Source` does not compile on CG, see `root-io-package.md`)
- Byte array stays mutable: `m(p)=32`; `m.indexOf(10,8)`, `m.lastIndexOf(84)` take Int/Char literals (cooperative `==`: Byte 10 equals Int 10)
- compare against Int codes, not char literals: `c==66` is 1 char shorter than `c=='B'`; `m(p)%b` with Int `b` works on Bytes
- `"SENW"indexOf c` / `"SENW"contains c` accept a Byte `c`
- tuple `var(p,k,b)=(1,2,3)` declares vars in one go; names must be lowercase (an uppercase name is a stable-identifier pattern -> `not found: value P`)
- `0 until 4` is a `Range` whose `.reverse` can be reassigned to the var; `0 to 3` is `Range.Inclusive` and fails
- char-class trick: `x%53==35` is true for `#` (35) and `X` (88) only; with `%54` only `#`: a breaker flag `b` in {53,54} (`b=107-b` toggles) replaces the `"X#".drop(b).contains` test
