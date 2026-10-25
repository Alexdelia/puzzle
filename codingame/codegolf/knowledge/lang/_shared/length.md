# Length

CG golf length = UTF-8 bytes of the code, trailing newlines excluded. Local: `agent/tool/count.sh`.

- probe 2026-10-19, Ruby on `temperature-code-golf`: code with `é€😀😀` and final `\n`: 75 bytes, 66 code points, 68 UTF-16 units, CG length 74
- 13 older solutions (Python, C, C++, Bash) all match `bytes - 1` with a final `\n`
- consequence: packing ASCII into multi-byte chars (`exec(bytes('...','u16')[2:])`) costs bytes, never saves
- final newline free: keep it or not, same length

## Stale char-counted rows above the byte-era floor (inference, verify per puzzle)

Verified facts: CG counted *characters* until at least 2022 (forum topic 194246 documents
the UTF-16 packing trick: `eval'...'.encode('UCS-2BE').b`, one source char carrying two
payload bytes), and the 2026-10-19 probe above shows bytes now. What is **not** verified:
when CG switched. Evidence from power-of-thor (2026-10-25, its puzzle knowledge + journal):
every leaderboard row below the current-region byte scores was created before 2026-10-19,
and the first rows created after the probe sit 20-40% higher (bash 51, Python 60, Perl 54,
PHP 74 on 2026-10-19..25). Treat a top far below the byte-era cluster as a possible stale
char-counted row: the recorded `criteriaScore` is bytes *at submission time*, not rescaled.
Packing only pays for payloads above ~55-60 bytes, so a stale low top is usually a much
longer program, not a shorter exploit. Do not assume it is reachable in bytes; check
creationTime against the probe date before spending submits.
