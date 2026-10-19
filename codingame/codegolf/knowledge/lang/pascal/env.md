# Pascal runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- Free Pascal 3.2.2, x86_64 Linux, binary runs as `/tmp/Answer`
- default mode FPC (no objfpc/delphi/tp/iso define): `Integer` is 2 bytes, `String` is shortstring (256 bytes, `{$H-}`); declare `ansistring` or add `{$H+}` for long strings
- `{$IFOPT}`: R-, Q-, I+, S-, C- (no range/overflow checks, IO checks on, asserts off)
- C operators `+=` `-=` work without `{$COPERATORS ON}`; `goto` with `label` works without `{$GOTO ON}` (both verified on CG)
- `for c in s do` over a string works
- stderr shown in test-game `output`, not judged
- local: `agent/tool/lang/pascal/build.sh` (plain `fpc`, nixpkgs fpc 3.2.2) reproduces all of the above
