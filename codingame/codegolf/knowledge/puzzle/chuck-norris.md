# Chuck Norris (Unary)

pretty_id `chuck-norris-codesize`, leaderboard `chuck-norris-codesize`, io.

## Rules

- 1 line, `0 < N < 100` ASCII chars, no CR; some tests have no final newline
- each char as 7 bits, runs of same bit: `0 <0*len>` for 1s, `00 <0*len>` for 0s, blocks separated by single spaces
- io limit 6000 ms per test
- all test and validator chars are printable ASCII 32..126 (validator 02 = single space, validator 04 = every printable char): 7 bits exact, no char >= 128, so `c>10` / `c>31` stops at newline and EOF
- output: no leading or trailing space allowed (see `knowledge/lang/_shared/output-compare.md`), final newline free

## Shortest known

Two families, pick by what the language makes cheap.

### A. Regex two-pass (Perl, Ruby, PHP, JS, sed, bash, any cheap global regex replace)

1. bit string: replace each char by its 7-bit binary (`sprintf"%07b",ord`, `"%07b"%ord`, `padStart(7,0)`)
2. `0+` runs -> ` 00 ` + run, THEN `1+` runs -> ` 0 ` + run. Order matters: the zeros pass first, so the zeros it inserts in headers are never re-matched; the ones pass inserts no 1
3. translate every `1` to `0` (`y/1/0/`, `tr`)
4. drop the first char (always the leading space of the first header)

Why short: no per-run length arithmetic, no lambda, no state. Needs: global regex replace with whole-match reference (`$&`, `\0`, `$&` in JS), char translate.
Measured: Perl 76 with `#!perl -p` shebang (perl honors switches on the `#!` line), 78 with `$_=<>`/`print`; 14/14 with local perl (not the official referee, no `CG_PERL` in shell).
Newline: `.` does not match `\n`, so the input's final newline survives untouched to the end of output, free.

One-pass beats the two-pass: `s/(0)+|1+/ 0$1 $&/g` (the group holds `0` only on zero runs, stays empty on one runs, so the header is `0` + group), then translate and drop the first char. Perl: 11 bytes under the two-pass, verified 14/14 (bash cell, `perl -pe`, 67). Same pattern in any flavor where an unmatched group substitutes as empty (`\1` in Ruby/PHP/sed, `$1` in JS), unverified there.
Perl bit step: `s/./sprintf"%v07b",$&/ge` (see `knowledge/lang/perl/sprintf-vector-flag.md`).
Perl core with both: 57 bytes. Bash top 64 and perl top 65 suggest a 54-byte perl core; not found (tries listed in the bash cell journal).

Variant rejected: one-pass backref `(.)\1*` -> `" 0".0 x!$1." ".0 x length$&` with `/e`: Perl 85, longer.
Variant rejected: one-pass `(.)\1*` -> ` 0$1 $&` then fix the `01 ` header: +3 over two-pass.

### B. Per-bit loop with previous bit (C-likes, Python, any language where regex costs an import)

For each char, for each of its 7 bits b (MSB first), with p = previous bit:
- b == p: emit `0`
- b != p: emit ` ` + header(b) + ` 0`, where header(b) = `"00 "` offset by b (`"00 "+b` in C, `'00 '[b>'0':]` in Python): `"00 "` for 0, `"0 "` for 1
- start with p = a third value (2 in C, `''` in Python) so the first bit is a boundary; drop the leading space once: C `" %s0"+(p>1)`, Python accumulate then `o[1:]`

Identity used: header(b) is a suffix of `"00 "` of length 3-b, so one string literal plus offset replaces a conditional.
C needs `c>>i&1` for `i=7..1`, i.e. `for(i=7;i--;)`, and stops on `c>10` (newline or EOF).
C trap: if b is assigned inside one printf argument and read in another, gcc evaluates args right to left: assign b in the LAST argument (assigning in the first argument gave 0/14).
Measured: C 103, Python 101 (nested `for c`/`for b in f'{ord(c):07b}'`, beats a single loop over `''.join(...)` by 5). Both 14/14 on the referee.

Python string compare trick, verified on paper, not shorter in Python: init p to `'00'`; then `b<p` iff a 0-run starts, `b>p` iff a 1-run starts (`'0'<'00'<'1'`), lets the header pick by comparison without a boundary test.

### Rejected

- pure str.replace at boundaries (`01`->`0 0 1`, `10`->`1 00 0`, then `1`->`0`): interior correct, but the first header needs the complement of the first bit prepended, which costs more than the regex import it saves (Python estimate 120+, not measured)
- Python `re.findall('0+|1+')` + per-run format: 119; two-pass `re.sub` in Python: 133 (import and `\g<0>` cost)
- packing bits through one big int (`n=n<<7|ord(c)`, `bin(n)[3:]`): longer than per-char format in Python (counted on paper, not run)
- `unpack"B*"` in Perl: 8 bits per char, removing the leading 0 of each byte costs more than `s/./sprintf.../ge`

### Gap to leaderboard (2026-10-19)

Tops: bash 64, perl 65, php 73, ruby 71, python 89, c 85, javascript 95. Our method lengths above are 11 (perl), 12 (python), 18 (c) chars over the top: the lang workers must find language tricks; the top Python 89 and C 85 likely use the per-bit loop (B) with tighter emission, unverified.

### C cell (2026-10-20)

C 101 (#29 on CG, top 85), verified 14/14: `main(c,p,b,i){for(p=2;(c=getchar())>10;)for(i=7;i--;p=b)b=c>>i&1,printf(" %s0"+(b-p?p/2:3),"00 "+b);}`. Format offset `p/2` (1 only when p=2 = first bit) beats `p>1`; assigning b in a comma expression avoids the arg-order trap.
Rejected: one literal with `\0` separators (" 00 \0 0 ") so `printf("%s0",L+f(p,b))`: needs linear f like `5*p-3*b+3` with 19-byte literal, longer; exhaustive expression search up to length 9 found no f. `%0*d` zero-width header: same length. Gap to 85 unexplained.

### Python cell (2026-10-20)

Python 101 (was 119), 14/14: `p=o=''` / `for c in input():` / ` for b in f'{ord(c):07b}':o+=(b!=p)*(' '+'00 '[b>'0':])+'0';p=b` / `print(o[1:])`. Every bit appends `0`; a boundary prefixes ` `+header, so no ternary (the `b==p and'0'or...` form is 105). Top 89 not found.

### Javascript cell (2026-10-20)

JS 127 (#110 on CG, top 95), 14/14, one-pass regex: `print([...Buffer(readline())].map(c=>(c+128).toString(2).slice(1)).join``.replace(/(0)+|1+/g,' 0$1 $&').replace(/1/g,0).trim())`. `$1` is empty on one runs in JS (verified). `.trim()` drops the leading space, 2 under `.slice(1)`; `Buffer` bytes are numbers, 2 under `charCodeAt()`. `+128` then `slice(1)` pads to 7 bits (same length as `padStart(7,0)`).
Rejected: per-bit `.replace(/./g,(b,i,s)=>...)` with previous-bit test, about +12. Gap to 95 unexplained.

## Hidden validators

9 validators mirroring the 9 IDE tests 1:1 (titles from CG's report, weight 250 each); 8 recovered exactly in `puzzle/chuck-norris/referee/hidden/01-07,09`, details, sources and the V8 partial facts in `puzzle/chuck-norris/referee/hidden/facts.md` (2026-10-24).
- 1 `G`, 2 `GG`, 3 `#`, 4 `Chuck Norris' keyboard has 2 keys: white space and 0.` (1-4 end with `\n`); 5 `ABCDEFGHIJKLMNOPQRSTUVWXYZ`, 6 every non-letter char 33..126 in ASCII order (42 chars), 7 `Chuck Norris can kill two stones with one bird.` (5-9 have no final newline)
- 8 "qwerty": 19 chars `q??w?e?r?t?y?u?o?p?` (letters exact, lowercase, no `i`), the 10 `?` are ASCII punctuation (no digit, space, letter), not from test 08's symbol set nor the shift-number row, unknown; 9 "Infinity": `999 times, CN counted to infinity` (exact)
- no single-space validator on CG (the "validator 02 = single space" in Rules is our synthetic one); a solution must handle both final-newline cases
- length separates all validators, but hardcoding is pointless: outputs (30-700 chars) are longer than any general solution

## CG rejections not caused by the rules (2026-10-21)

- scala 147 scored 0 on CG though it passed 9/9 locally: `io.StdIn` does not compile on CG (`knowledge/lang/scala/root-io-package.md`). Referee rules were right. The local pass came from a dev shell with a stale `CG_SCALA` (store path from before the io.circe stub). With `nix build .#scala` the same file fails locally with CG's error text, and `scala.io.StdIn` passes 9/9. If a scala cell passes locally with `io.X`, rebuild the toolchain before submitting
- referee: 10 validators (06-10 added: doubled `%%` with `\n`, digit-first 31 chars, 99 chars with and without `\n`, short sentence); negative checks: missing `[1:]` 0/9, a 7 s sleep times out at 6000 ms; bench suite 691 ms, referee CPU <= 0.4 ms per case

### OCaml cell (2026-10-22)

OCaml 82 (#2, top 79), CG accepted: `Sys.command{|perl -lpe'<56-byte -lp core of the perl cell>'|}`. Native OCaml (recursion over `input_byte`, previous-bit param, separator param) measured 179: hopeless. Top 79 = 13 + `perl -lpe''` 11 + 55, or 13 + 2 + bash top 64: the gap is the perl core, same 2-3 bytes the perl and bash cells miss. A 53-byte `-lp` core without `"` (or 54 with one `"` pair via `q()`) would tie.
