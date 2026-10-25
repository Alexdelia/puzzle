# Ruby env on CG

Verified 2026-10-19 with test games on chuck-norris, CG output identical to local nix (nixpkgs 22.05 `ruby_3_1`):

- `RUBY_VERSION` 3.1.2, patchlevel 20, release 2022-04-12, `x86_64-linux`: so 3.1 syntax applies (endless methods `def f=x`, numbered block params `_1`, hash shorthand `{x:}`, anonymous block `&`, rightward `=>` match) and `it` (3.4) does not; these follow from the version, not individually run on CG
- bignums native (`2**64` exact), `0.1` prints `0.1`
- `Encoding.default_external` and `__ENCODING__` UTF-8
- `require 'prime'` and `require 'set'` succeed (prime is a bundled gem in 3.1)
- 8 entries in `$LOAD_PATH` both sides
- only drift: CG ruby built with YJIT (`RubyVM::YJIT` defined, disabled), local nix build has no YJIT: irrelevant for output
- stderr shows in test-games `output` but is not judged
- local launcher startup ~75 ms per run (interpreter boot + rubygems)

## Determinism of built-in hashes (verified 2026-10-25)

`String#hash`/`Object#hash` are seeded per process: `"2\n1 2\n2 3\n".hash%9` gave
4, 8, 7 on three runs of the same Ruby 3.1.2, so hardcoded keys must never use
`hash`/`object_id` (they differ for every validator run).
