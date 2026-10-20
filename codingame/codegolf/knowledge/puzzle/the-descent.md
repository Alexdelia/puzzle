# The Descent

pretty_id `the-descent-codesize`, leaderboard `the-descent-codesize`, interactive.

## Rules (as referee)

- no init input. each turn: 8 lines, one height per line (0..9), final newline after each
- output one line: index 0..7 of the mountain to shoot
- altitude starts 10. per turn: altitude -1, then crash if any CURRENT (pre-shot) height > altitude, else shot applies
- each mountain has a hidden list of heights it takes on successive shots, ending at 0 (test 5: 8 7 6 5 4 0; strong mountains: 6 4 2 0, 7 4 0). shot on a 0 mountain: nothing
- win: all heights 0 after a shot. always shooting the current max survives every test and validator (equal heights: any of them)
- game ends at most ~9 turns: altitude caps total shots
- tests: 1 descending 9..2, 2 scattered, 3 and 4 strong mountains (several shots each), 5 one mountain at index 3

## Output parsing (probed on CG 2026-10-19)

- Java `Integer.parseInt` semantics, then range 0..7
- accepted: `+0`, `01`, `-0`, `2\r`, long runs of leading zeros (`000000005`)
- rejected: `0 ` (trailing space), ` 0` (leading space), `-3`
- consequence: stray digit `0` written to stdout before the answer on the same line is harmless, e.g. Python `input(0)` prompt writes `0` to stdout (stdin not a tty), 8 reads per turn prefix the answer with `00000000` and CG still accepts it

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 150 ms (0.14 s pass, 0.16 s timeout); statement says 100 ms, wrong

## Input quirks

- heights single digit: comparing raw lines as strings (or first byte) orders them like integers
- mountains with height 0 sent as `0`, never blank

## Shortest known

Method: per turn, argmax of the 8 heights. Nothing simpler survives: test 1 (9..2 descending, one shot per height) forces a max shot every turn, so no fixed order, no "first nonzero", no threshold rule. Freedom left: tie side (first or last max both pass, verified with `>=` scan) and how argmax is written.

Forms, shortest first where the language has them (all verified 15/15 on local referee unless noted):

- key-on-index: `max over 0..7 keyed by reading one line`, no array, no index lookup. Needs a max-by whose key runs exactly once per element, in order. Python with `key=lambda _:input()` on `range(8)` reached 49 (ties top 49, 236 players). Ruby `max_by{gets}` on `0..7` inside `loop` counts 27 = top (not run locally, no ruby toolchain in this shell). Candidates (unverified): Kotlin/Groovy/Scala `maxBy`, Clojure `max-key` (check it does not re-call the key), C#/F#/D/Haskell equivalents
- raw line compare: heights are one digit, so lines compare as strings in int order; skip the parse (`gets`, `input()`, `readline()` raw)
- combined key: `max(h*8+i)%8`, or string `line+index` then last char (Python 55, longer there but may win where index-of is costly). Verified in C with `h*8+i` then `%8`
- running scan: reset `m` each turn, `h>m` (first max) or `h>=m` (last max)
- shell pipeline: per turn take 8 lines, number them from 0 (`nl -v0`), sort on the height field, keep the last line, cut the index digit; bash total 55 with `while :;do ...;done` (verified with local bash, not CG bash). `for((;;)){ ...;}` is 1 shorter. Over-reading tools (`head`) are safe: the referee sends the next turn only after the answer
- bash top is 39, perl 36: bash 39 is likely a call to another interpreter on the CG bash image (`perl -e`, `ruby -e`, awk); which ones exist there is unverified

Shared plumbing:

- infinite loop, die at EOF: game ends after at most 9 turns, crash after last output passes (`knowledge/lang/_shared/crash-exit.md`)
- flush every turn: C `printf` without `fflush(0)` times out at turn 1 (verified). Same trap in any language with block-buffered stdout on a pipe (Perl `$|`, etc.)
- Python `input(0)` / `map(input,[0]*8)` prompt writes `0`s before the answer, harmless. `key=input` is NOT: prompts `0`..`7` give `012345675`, rejected (verified)

Rejected:

- Python `max(enumerate/zip(...))[1]` 51-53, `a.index(max(a))` 51-52: longer than key-on-index
- Python `os.read(0,99)[::2]` chunk read: needs `import os`, about 61
- turn-monotone key (`80t+8h+i`) to avoid resetting the running max: index math costs more than the reset
