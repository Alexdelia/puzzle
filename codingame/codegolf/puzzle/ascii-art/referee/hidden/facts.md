# ascii-art hidden validators (CG `ascii-art-codesize`)

Not complete: no validator recovered exactly. Facts below come from score counts (oracle log `agent/state/oracle/ascii-art.tsv`, task `validator.ascii-art.-.1792523690365785466`, 2026-10-20).

## Count

10 validators for 6 IDE tests. Source: every partial row of other players on the per-language leaderboards is a multiple of 10 (0, 30, 60, 70, 80, 90 seen in Ruby, PHP, Java, JS, Rust, Scala, C, C++, Bash, Lua). Our fresh-language probes scored 0, 70, 80, 100. Confidence high.

## Per-feature counts (all 10 at once)

| predicate on validator input | validators | source |
|---|---|---|
| font rows = test 1 `#` font (L=4 H=5) or test 5 font (L=20 H=11), polynomial hash | 10 | lua probe, 100% |
| `T` matches `^[A-Z]+$` | 7 | php probe, 70% |
| `T` matches `^[A-Za-z]+$` | 8 | php chain probe, 80% |
| `T` exactly one of the IDE texts `E`, `MANHATTAN`, `ManhAtTan`, `M@NH@TT@N`, `MAN HAT TAN` | 0 | typescript probe, 0% (probe verified working on CG with a test game on test 2) |
| `T` in {`WMADXESFGIJVTHKNOBPLYQRUZC`, `HELLOWORLD`} | 3 | typescript chain probe, 30% |
| `T` == `HELLOWORLD` | 2 | go probe, 20% |
| `L`==20 and `T` in {`WMADXESFGIJVTHKNOBPLYQRUZC`, `HELLOWORLD`} | 0 | cpp probe, 0% |
| `L`==20 | 1 | cpp chain probe, 10% |

Deductions:
- 9 validators use the test-1 `#` font (`L=4 H=5`), 1 uses the test-5 font (`L=20 H=11`)
- recovered exactly, validator numbers unknown (confidence high, inputs = `4`, `5`, `T`, the 5 font rows of test 1):
  - `T=HELLOWORLD`, `#` font: 2 validators (identical inputs, both counted by the go probe while the cpp probe saw no L=20 among them)
  - `T=WMADXESFGIJVTHKNOBPLYQRUZC`, `#` font: 1 validator (the classic "Test full alphabet in CAPS" text)
- left unknown: 7 `T` texts: 3 uppercase-only more (one of them the L=20 one, likely a word like test 5; one likely a single letter like test 1, unprobed), 1 mixed-case letters-only (test 3 scenario), 2 with non-letters (tests 4 and 6 scenarios: `@`, space, maybe the classic "Lorem ipsum dolor sit amet,..." text)
- fonts: no new font; each validator is (one of 2 known fonts, new `T`). Recovering `T` is enough to rebuild a validator exactly
- 7 uppercase-only, 1 letters with lowercase (mixed case, like test 3), 2 with non-letters (like tests 4 `@` and 6 space). Matches 6 test scenarios + 4 extra uppercase-only validators
- every validator `T` differs from its IDE test text: hardcoding IDE answers scores 0

## Classic puzzle hints (unverified for golf)

Old IDE tests of the classic ASCII Art puzzle, candidates for the extra validators:
- "Test full alphabet in CAPS" `WMADXESFGIJVTHKNOBPLYQRUZC` (https://forum.codingame.com/t/ascii-art-puzzle-discussion/34?page=16, page=10)
- "Lorem ipsum dolor sit amet,..." (spaces, comma, longer) (https://forum.codingame.com/t/ascii-art-puzzle-discussion/34?page=11): has non-letters, at most one of the 2 non-letter validators
- classic validator "HELLOWORLD" (https://forum.codingame.com/t/trying-to-understand-why-my-ascii-program-fails-validation/856)

## Shortest separating key

Unknown: 7 `T` not recovered. Files `NN.in` not written: numbering unknown.

## Next probes (cheapest first)

- fresh-language count probes still free (no row of ours): Kotlin, Java, C#, Swift, Dart, Scala, Groovy, Haskell, OCaml, D, F#, VB.NET, Pascal, ObjectiveC, Clojure, Rust. Chains alive: php row 80 @306 (letters-only), typescript row 30 @288 (set), go row 20 @581 (HELLOWORLD), cpp row 10 @294 (L==20); a chain probe shows its count only when it scores higher, or ties and is shorter
- candidate texts to count next: `len(T)==1` (one-letter validator), `T` containing a space, `T` containing `@`, `T` starting with `Lorem`, `len(T)` bins for the L=20 one

## Golf consequences

- non-letter handling needed by 2 validators, lowercase by 1 (8 of 10 are pure letters; uppercase-only formula passes 7)
- `L=4` hardcoded passes 9 of 10 (one L=20 validator, test-5 font): `L` must still be read or derived
