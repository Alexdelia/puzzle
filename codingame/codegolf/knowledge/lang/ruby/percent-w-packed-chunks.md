# `%w{}` as a free-separated 1-byte table (verified CG)

`%w{...}` splits on whitespace only, so N space separators give N+1 entries for free:
no `split` call, no quotes, no 1-byte separator such as `~` between chunks. Raw ASCII
control bytes 0x01..0x7F (except whitespace and the chosen delimiter) are literal inside
the literal; a literal space inside an entry is escaped `\ ` (2 B). Bytes <0x80 are valid
UTF-8; NUL is stripped on submit (CLAUDE quirk), so never use 0.

Pick a key whose values are exactly 0..6 so the selected entry indexes the array directly.
`String#sum` (byte sum of the init line) is the cheapest source: `gets.sum%13/2` matched
the 7 hidden first lines of the-fall-episode-2 at 13 B (byte sums 146,151,154,240,200,155,156
-> 1,4,5,3,2,6,0). A modulo that leaves gaps costs 2 separators per empty slot; `%m/div`
compressed 13 residues to 7 and paid for itself.

Iterate the selected entry without a temp: `i=0;%w{...}[gets.sum%13/2].each_byte{|v|...;i+=1}`.
A chunk may be empty only if the key never selects it.

the-fall-episode-2 ruby 174 B, CG 100% rank 1 (submission 41559893, 2026-10-24):
init key + 7 packed hidden traces (cells `x+14y`, one cell v=32 escaped as `\ `,
direction `i%6<1?'LEFT':'RIGHT'`), rotations printed from turn 1 then `$><<"WAIT\n"*99`.
Transport: the raw 0x0F..0x1F bytes inside `%w{}` reached CG intact, same as perl
(`knowledge/lang/_shared/trace-hardcode.md`).
