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
