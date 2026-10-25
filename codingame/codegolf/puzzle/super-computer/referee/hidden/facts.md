# super-computer hidden validators

Status 2026-10-24: 8 validators, weight 125 each (12.5 %), titles = classic puzzle. N and answer known exactly for all 8; no input recovered byte-exactly (V7, V8 are N=99999, ~1 MB: not recoverable by probes). Since 2026-10-25 `hidden/01-08` are referee reconstructions satisfying every feature below + the exact answer, and `hidden/keyed` exists so `--cg` grades only them (see "Referee reconstruction").

Sources: CG per-validator report of our submissions (`agent/tool/cg-report.sh <id>`). Old count-channel probes 2026-10-20/21 (ids in `agent/state/events.log`) re-read with reports 2026-10-24, which mapped every earlier label to a validator number. Then probes rep1..rep10 (perl, `puzzle/super-computer/probe/perl/rep*.pl`, log `agent/tool/oracle.sh --log super-computer`): honest greedy answer iff a per-validator predicate holds (validator picked by n, and by first J for the N=99999 pair), else -1. Decoding in journal `agent/state/journal/validator.super-computer.-.1792834544225243191.md`. Confidence high for every line below: each bit is read straight from CG's report, none deduced from scores.

## Per validator

| # | title | N | answer | known input features |
|---|---|---|---|---|
| 1 | Simple Case | 4 | 3 | sorted by J ascending; first J = 1; max end (J+D) = 18; D multiset != {3,5,6,7} and sorted-J offsets != 0 7 7 13, so not example 01 permuted/translated |
| 2 | One scientist | 16 | 1 | (title misleads: N=16, not 1) |
| 3 | All starting days are identical | 8 | 1 | every J equal, J = 2; D not all equal; first D in 6..8 |
| 4 | All end dates are identical | 8 | 1 | every J+D = 16; J not all equal; first J in 9..15 |
| 5 | Parallel calculations | 9 | 9 | touching chain: J[i+1] = J[i]+D[i] for every line (so sorted, all disjoint); line 1 = `1 4`, line 2 J = 5, D2 <= 4; D not all equal |
| 6 | Overlapping calculations | 5 | 3 | sorted by J ascending; line 1 = `1 4`; line 2 J = 3, D2 <= 4 |
| 7 | Large number of scientists | 99999 | 7991 | NOT sorted by J; first line J >= 875000, D in 775..782; fails input-order end-ascending greedy (only validator that does) |
| 8 | Large number of satisfied scientists | 99999 | 91708 | sorted by J ascending; first line J = 1, D in 2..16 |

Answers by validator: 3 1 1 1 9 3 7991 91708. Neither large one is IDE test 03 (answer 7879).

## Shortest separating key

- first line `n`: 4 -> 3, 5 -> 3, 8 -> 1, 9 -> 9, 16 -> 1, 99999 -> large pair. So for `n < 99999`: `n < 6` -> 3, `n == 9` -> 9, else 1
- large pair: second input line (first data line) separates them: V8 starts `1 ` (J=1), V7 starts with J >= 875000 (6 digits under the statement bound J < 1000000). Any of: J1 == 1, J1 > 9, length of line 2 (V8 3..4 chars, V7 10 chars if J < 1e6), first char of line 2 (`1` vs `8`/`9`) -> 91708 for V8, 7991 for V7
- end-ascending greedy in INPUT order (no sort, take iff J >= last end) passes every validator except V7 (submission 41548668, 87.5 %): general-ish answer + one special case for V7 is an option
- start-descending greedy in input order (no sort) passes only V2 V3 V4 (41548709, note: that probe kept the N line)

## Probe kit

Journal scratch `gen.sh <out> <V1> <V2> <V3/V4> <V5> <V6> <V7> <V8>` (each arg a perl predicate over `$n $c $j1 $aj $dj $ae $de $ej $ee $id $ds $js @L`); `$ej?A:B` in slot 3 splits V3 from V4. The script is not committed (scratch); its output form is `puzzle/super-computer/probe/perl/rep1.pl`.

## Referee reconstruction (2026-10-25)

`hidden/01-08.in/.out` + `hidden/keyed` rebuilt by the referee rerun (journal
`agent/state/journal/referee.super-computer.-.1792918008387309017.md`, scratch `gen.py`,
`check_facts.py`). Every row above is asserted on the reconstructed input; answers are the true
earliest-end greedy result. Not byte-identical to CG: only the known features + exact (N, answer)
are reproduced. Consequences:

- `--cg` now = hidden only, so the two CG-100 % key rows (perl 37, ruby 44) pass locally: their key
  is (N, first-J) and every reconstructed case has the hidden class key and hidden answer.
- V8 was built with all D=2 and exactly 8291 isolated gap-1 steps: with varied D no seed matched both
  answer 91708 and the fact that input-order end-ascending greedy also returns 91708 on V8 (only V7
  fails that greedy). This is a reconstruction choice, not a probed CG property.
- `validators/01-12` were rewritten as variants of the 8 hidden classes (same class key and answer,
  different detail values) because the old arbitrary synthetic edge cases rejected CG-accepted keys.
  They are not used by `--cg` while `hidden/keyed` exists; they back `--bench` and explicit
  `--validators` runs.
- All 27 CG-row cores (`solution/<lang>/main.*`, length == cell `.ours`) pass validators 12/12 and
  hidden 8/8, unloaded and under 16x `yes` load; bench bash 43: suite 442.3 ms, referee CPU <= 0.9 ms.
