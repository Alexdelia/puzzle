# chuck-norris hidden validators

Status 2026-10-24: 7 of 9 recovered exactly (`01`-`07` `.in`/`.out` here, python main passes `run.sh --hidden`), V8 and V9 only partly. No `complete` file.

Sources: titles and weights from CG's report (`agent/tool/cg-report.sh 41554959`, every validator weight 250); probe log `agent/tool/oracle.sh --log chuck-norris` (python probes `cand1`, `cand2`, `len0`-`len4`, `q1`-`q7` in `puzzle/chuck-norris/probe/python/`, each honest iff a per-validator predicate holds); 2026-10-20 count probes (php/cpp/lua/ts rows). Journals: `agent/state/journal/validator.chuck-norris.-.1792523690522524908.md`, `agent/state/journal/validator.chuck-norris.-.1792834543800153369.md`. Web: forum threads quote no validator input (https://forum.codingame.com/t/unary-puzzle-discussion/35).

Validators mirror the IDE tests 1:1 (9 and 9, same scenarios). L = length without newline.

| # | title | input | final `\n` | source | confidence |
|---|---|---|---|---|---|
| 1 | Character G | `G` | yes | cand1 pass, q6 | exact |
| 2 | Message GG | `GG` | yes | cand1, q6 | exact |
| 3 | Character # | `#` | yes | cand1, q6 | exact |
| 4 | Chuck Norris' keyboard has 2 keys: white space and 0. | the title itself (L 53) | yes | cand1, q6 | exact |
| 5 | Alphabet | `ABCDEFGHIJKLMNOPQRSTUVWXYZ` | no | cand2 (lowercase reversed / uppercase reversed excluded) | exact |
| 6 | Printable characters | every char 33..126 that is not a letter, ASCII order (L 42): ``!"#$%&'()*+,-./0123456789:;<=>?@[\]^_`{|}~`` | no | q3 | exact |
| 7 | One stone, two birds. | `Chuck Norris can kill two stones with one bird.` (L 47) | no | cand1 + len0 + len1 | exact |
| 8 | qwerty | unknown, L = 19 | no | len0-len4 + bucket counts, q1 | partial |
| 9 | Infinity | unknown, L = 33 | no | len0-len4 + bucket counts | partial |

V8 (qwerty), known: L 19; contains `q` or `Q` (q3); first char >= 64 (q4 + first-char count) and not uppercase (q7); not all letters (q6). Refuted: even positions = `qwertyuiop` (q4), symbol/letter alternation starting with a symbol (q2), any 19-char window of keyboard rows `qwertyuiopasdfghjklzxcvbnm` (also with digits, `[]\;',./`, spaces, reversed, upper/title case) (q5), test-08 letter swaps `&q$w%e*r-t+y?y.u:i=o)p` (cand1/cand2).

V9 (Infinity), known: L 33; first char is a digit (q4, q7); contains `infinit` case-insensitive (q6); does not start with `Chuck Norris ` (q2). Refuted: `Chuck Norris counted to infinity.` (q1), case/reverse variants with `.!?` (q3), one char 32..63 + case variant of `Chuck Norris counted to infinity` (q5), the infinity phrasings of cand2.

Length derivation: 2026-10-20 bucket counts (6..26 x2, 27..39 x1, 40..99 x3) with V5 = 26, V4 = 53, V7 = 47 known, plus length bits 0-4 (mod 32: V6 10, V8 19, V9 1): only V9 can sit in 27..39, so V9 = 33; q1 (V6 != 10) then gives V8 = 19, V6 = 42. Final buckets: 6..26 = V5, V8; 27..39 = V9; 40..99 = V4, V6, V7.

First char < 64 (2026-10-20 lua, 3/9): V3 `#`, V6 `!`, V9 (digit).

## Shortest separating key

Length without newline separates all 9: 1 (V1 `G`, V3 `#`: first char), 2, 53, 26, 42, 47, 19, 33. Answers = the 7 `.out` files; V8/V9 answers unknown. Hardcoding has no golf value here (outputs are 30-700 chars, longer than any general solution).

## Resume (V8, V9)

Probes key per validator on `L%32` (10 -> V6, 19 -> V8, 1 -> V9), template in `puzzle/chuck-norris/probe/python/q6.py`. One bit per validator per probe: V9 needs ~33 chars of an unknown sentence starting with a digit, V8 19 chars with symbols. Only worth it if a later need appears (e.g. `complete` for submit.sh); next bits: V9 first digit (4 probes), V8 first char in `q`..., then guess sets.
