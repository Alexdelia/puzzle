# Temperatures

pretty_id `temperature-code-golf`, leaderboard `temperatures-codesize`, io.

## Rules (as referee)

- line 1 `N` (0 <= N < 10000), line 2 N ints in [-273, 5526], single spaces
- print the value with least `|t|`; tie `-t`/`t`: positive wins; N = 0: print `0`
- output compared line by line, trailing empty lines free, trailing space fails

## Input quirks (captured 2026-10-19)

- every test ends with `\n`
- test 6 (N = 0) is `0\n\n`: line 2 exists and is empty, so reading line 2 never hits EOF; split of it gives no tokens
- CG tests: 01 `1`, 02 all negative `-5`, 03/04 tie `-5`/`5` in both orders, 05 N=10, 06 N=0
- test 06 capture: `capture-io.sh` reports truncation (CG output strips trailing newlines of the stderr echo), file written by hand from a `repr` probe

## Timing (probed on CG 2026-10-19)

- io limit 6000 ms: sleep 5.5 s pass, 6.5 s timeout

## Tricks (verified on CG tests)

- tie-break toward positive without a branch: key `abs(t-.1)` (any epsilon in (0, 0.5))
- `key=abs` alone picks the first of a tie: fails tests 03 or 04 depending on order
- empty case: `split()or[0]` in Python yields the required `0`
