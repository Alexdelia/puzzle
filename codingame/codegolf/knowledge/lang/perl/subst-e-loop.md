# s///ge as the query loop, match vars as parsed ints

- `s/\d+ /CODE/ge` instead of `while(/(.+) (.+)/g){CODE}`: /e replacement is a block, statements and `for` modifiers allowed; `$_` inside still reads the whole slurped input. Saves 6+ bytes. Verified surface (5.34, CG 100%).
- `$&` = `"X "` numifies to X, `$'` = rest of input numifies to the next number (Y): `$&+$'*$w` with no captures. Read them before any inner regex match in CODE.
- header: `$w=s/.+\n.+\n//+$&` (s returns 1, `$&`="L\nH\n" numifies to L): width+1 and strip in 17 bytes.
- TRAP: unanchored `s/.+ /` on a slurped 1M-char grid backtracks over every long line: correct but times out; `\d+ ` is fast.
- `stmt for@k=LIST` iterates the assigned list (no growth); use `@k=LIST;stmt for@k` for a growing work list.

Source: surface perl task 1792518467322473131.
