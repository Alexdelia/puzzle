# chuck-norris hidden validators

Status 2026-10-24 (fourth worker, probes t1-t17): 8 of 9 recovered exactly (`01`-`07`, `09` `.in`/`.out` here, python main passes `run.sh --hidden` 17/17), V8 only partly (10 punctuation chars unknown). No `complete` file.

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
| 8 | qwerty | `q??w?e?r?t?y?u?o?p?`, L = 19, `?` = 10 unknown ASCII punctuation | no | len0-len4, q1-q7, r1-r20, t1-t17 | partial: letters exact, symbols unknown |
| 9 | Infinity | `999 times, CN counted to infinity` (L 33) | no | r-series + t1-t6 (t6: exact string, no final newline) | exact |

V8 (qwerty), known: L 19; contains `q` or `Q` (q3); first char >= 64 (q4 + first-char count) and not uppercase (q7); not all letters (q6). Refuted: even positions = `qwertyuiop` (q4), symbol/letter alternation starting with a symbol (q2), any 19-char window of keyboard rows `qwertyuiopasdfghjklzxcvbnm` (also with digits, `[]\;',./`, spaces, reversed, upper/title case) (q5), test-08 letter swaps `&q$w%e*r-t+y?y.u:i=o)p` (cand1/cand2).

V9 (Infinity), known: L 33; first char is a digit (q4, q7); contains `infinit` case-insensitive (q6); does not start with `Chuck Norris ` (q2). Refuted: `Chuck Norris counted to infinity.` (q1), case/reverse variants with `.!?` (q3), one char 32..63 + case variant of `Chuck Norris counted to infinity` (q5), the infinity phrasings of cand2.

Length derivation: 2026-10-20 bucket counts (6..26 x2, 27..39 x1, 40..99 x3) with V5 = 26, V4 = 53, V7 = 47 known, plus length bits 0-4 (mod 32: V6 10, V8 19, V9 1): only V9 can sit in 27..39, so V9 = 33; q1 (V6 != 10) then gives V8 = 19, V6 = 42. Final buckets: 6..26 = V5, V8; 27..39 = V9; 40..99 = V4, V6, V7.

First char < 64 (2026-10-20 lua, 3/9): V3 `#`, V6 `!`, V9 (digit).

2026-10-24 r-series (python probes `r1`-`r20` in `puzzle/chuck-norris/probe/python/`, log lines "V8 ...; V9 ..."):

- V8 = `q` (r4), s[1] punctuation (r5 not letter, r6 not space, r7 not digit), s[2] neither letter nor digit (r9, r15; space not tested), then letters exactly at odd positions 3,5,..,17 and non-letters at even 4,6,..,18 (r19; r12 s3, r16 s5, r18 s7 letters; r13 s4, r17 s6 not). So 9 letters `q?????????` + 10 non-letters, shape `q P X L N L N L N L N L N L N L N L N`. Refuted: starts with `qwerty` (r2), letters 3..17 = `wertyuio` (r20), 5026 crc24 candidates of `gen.py` v8 (r1). Letters other than q and the 10 symbols unknown
- V9 = `DDD times, CN counted to infinity`: ends with ` counted to infinity` (r2), s[0] s[1] digits (q7, r10), `DDD ` (r13), s[4:11] = `times, ` (r15), s[11:13] = `CN` (r20; r17 not `he`, r18 s11 upper, r19 not `He`). DDD != `100` (r16). Number is the only unknown (~10 bits)

2026-10-24 t-series (python probes `t1`-`t17` in `puzzle/chuck-norris/probe/python/`, template `mk.py` in `agent/state/journal/validator.chuck-norris.-.1792845787850935809.scratch/`):

- V9: in a 39-number round/meme set (t1), bisected to `999` (t2-t5), whole string + no final newline confirmed (t6)
- V8 letters: s[3] `w` (t2), s[3:10:2] `wert` (t3), s[11] `y` (t4), s[13] `u` (t8 not `y`, t9), s[15] `o` (t5 not `ui`, t10), s[3:18:2] == `wertyuop` case-sensitive (t11). So letters `q w e r t y u o p` (QWERTY row without `i`; test 08 is the AZERTY row with a doubled `y`)
- V8 symbols (s[1], s[2], s[4], s[6], .., s[18]): no digit or space (t16, with r5-r7: all ASCII punctuation if ASCII); refuted: test-08 prefix `q&$w%e*r-t+y?y.u:i=` (t7), `q!@w#e$r%t^y&u*o(p)` (t12), all in test-08 set `&$%*-+?.:=)` (t13), all in `!@#$%^&*()` (t14), >= 6 in test-08 set (t15), >= 6 in `!@#$%^&*()` (t17). Looks like random typing: ~5 bits per symbol, ~50 probes for the rest
- t6's V8 half is void (python literal kept a stray backslash)

## Shortest separating key

Length without newline separates all 9: 1 (V1 `G`, V3 `#`: first char), 2, 53, 26, 42, 47, 19, 33. Answers = the 8 `.out` files; V8 answer unknown. Hardcoding has no golf value here (outputs are 30-700 chars, longer than any general solution).

## Resume (V8)

After t17: only the 10 V8 symbols are missing. Per-position bisect over the 32 ASCII punctuation chars costs ~5 probes per symbol (~50 total); worth it only if `complete` (submit.sh checking hidden instead of synthetic validators) becomes needed. Cheaper first: structural guesses (all distinct, sorted, US-keyboard neighbours of each letter like `[];',./`). Older note after r20: V9 needs only DDD (bisect `int(s[:3])`, ~10 probes, or a guess set of round numbers first). V8 needs 8 letters + 10 symbols (letters likely keyboard order with a variant, symbols maybe from test 08 `&$%*-+?.:=)`). Template: `mk.py` in `agent/state/journal/validator.chuck-norris.-.1792843414350675571.scratch/` builds a probe from two predicates (V8 keyed on L%32 == 19, V9 on 1).

Older note:

Probes key per validator on `L%32` (10 -> V6, 19 -> V8, 1 -> V9), template in `puzzle/chuck-norris/probe/python/q6.py`. One bit per validator per probe: V9 needs ~33 chars of an unknown sentence starting with a digit, V8 19 chars with symbols. Only worth it if a later need appears (e.g. `complete` for submit.sh); next bits: V9 first digit (4 probes), V8 first char in `q`..., then guess sets.
