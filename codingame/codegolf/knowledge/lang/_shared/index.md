# Shared: CG facts valid in every language

- [length](length.md): length = UTF-8 bytes, trailing newlines free, unicode packing useless, verified
- [output-compare](output-compare.md): exact line match, trailing spaces and CR fail, missing final newline and ONE trailing empty line free, two or more trailing empty lines FAIL (local referee-core enforces it too), verified
- [io-time-limit](io-time-limit.md): io puzzle run limit about 6000 ms per test in Python, startup included; per language (Groovy ~1.9 s), verified
- [memory-limit](memory-limit.md): about 768 MiB peak RSS per test, over it = fail (Python killed silently, Haskell shown as timeout), local referee does not check it, verified
- [crash-exit](crash-exit.md): io puzzles ignore exit status, crash after output passes, verified
- [best-kept](best-kept.md): CG keeps best submission per language, longer submit harmless but unconfirmable, verified
- [system-wrapper](system-wrapper.md): `os.system("bash -c ...")` runs bash on CG, interactive io works; cell length = wrapper + shortest core, verified on Python; TRAP: spawned `perl` locally = host 5.42, CG = 5.34 (variable-lookbehind bug), test with `PATH=$CG_PERL/bin:$PATH`
- [hidden-validators](hidden-validators.md): hardcode/exploits allowed (user rule), `submit.sh --probe`, best-kept makes each probe a 1-bit oracle when shorter than our row, local validators synthetic; first submit in a no-row language reads passed/total via the partial row score (or sum of weights: scores can be weighted)
- [orphan-child-kill](orphan-child-kill.md): child of an exited main process is killed ~0.3-0.5 s later on CG (wait for it or exec); local referee does not model it, verified
- [pipe-block-keepalive](pipe-block-keepalive.md): open-loop interactive bot, repeat the last answer 1e5 times so the write blocks on the full pipe instead of reading stdin to EOF, verified on CG (Mars Lander 3 perl)
- [interactive-early-exit](interactive-early-exit.md): print all turn answers at once then exit; CG reads the buffered stdout and ignores writes to the dead player, no keepalive needed (verified the-bridge-episode-2 ruby 152 CG 100%); shared referee-core still rejects it locally, the-bridge-episode-2, the-fall-episode-2, the-fall-episode-3 and vox-codei-episode-2 got a tolerant runner 2026-10-25
- [floordiv-step-chain](floordiv-step-chain.md): `i<T1?R1:i<T2?R2:0` = `(T1-1)//i*(R1-R2)+(T2-1)//i*R2` while i > T2/2, verified Lua mars-lander-episode-3
- [abs-tie-comparator](abs-tie-comparator.md): sort comparator `a*a-b*b-a` = min |t|, positive wins tie, nonzero ints only (-4 Perl vs `||$b-$a`), verified Perl + Node
- [self-exec-glob](self-exec-glob.md): `system("...;./*")` loops by re-running /tmp/Answer (cwd /tmp, glob sorts Answer first), 3 B, verified OCaml; other boxes unverified
- [trace-hardcode](trace-hardcode.md): with `referee/hidden/complete`, hardcode the reference action traces keyed on the init input, print them early (rotations can move earlier), then the no-op; probe-only, verified the-fall-episode-2 perl 187 rank 1
- [leaderboard-creation-time](leaderboard-creation-time.md): leaderboard rows carry `creationTime` ms: check a top is current before assuming it is stale, verified on CG
- [lookup-perfect-hash](lookup-perfect-hash.md): brute-forced cheap perfect hash of recovered validator inputs, no-op slots for AWK split, probe-only, verified blunder-episode-1 ruby 172
