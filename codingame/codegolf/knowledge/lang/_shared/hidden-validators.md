# Hidden-validator exploits

User rule (2026-10-20): hardcoding and exploits are allowed and part of the challenge; on some puzzles finding the hidden exploit is the whole puzzle.

## Per-validator report (2026-10-24)

- `submit.sh` (and `oracle.sh` through it) reads CG's report of every submission it sends: pass/fail of each hidden validator by number, title and CG method name, plus its weight. Printed as `report 50.0%: 1 Avoiding rocks [Validator_1] fail, 2 Rock interception [Validator_2] pass, ...`, written in the probe's `probes.tsv` row and in `agent/state/report/<submission-id>.json`; `agent/tool/cg-report.sh <submission-id>` reads it again for any of our submissions
- the report holds whatever our row: a probe longer than our row, or under 100%, still answers per validator. Validator count, titles and weights come with the first submit; no score-grain deduction needed
- one probe = one bit per validator: answer correctly iff the input satisfies a predicate, garbage otherwise, and the report says which validators satisfy it. Bisect a value per validator in parallel (each probe halves every validator's range at once), test the candidate transform of test k against validator k, read one answer bit per validator
- a 100% report on a submit shorter than our row, with the row unmoved, is a lost leaderboard update (`submit.sh` says so): resubmit the same code, never mark the referee suspect
- when `submit.sh` prints `report unavailable` (broker down or CG session expired, the user fixes it), the row oracle below is the fallback

## Row oracle (fallback without report)

- CG scores a submission on its hidden validators only, not on the IDE tests; tops far below any general method usually hardcode answers keyed on validator inputs or rely on weak validators
- local `validators/` are synthetic (written by our referee tasks): an exploit fails them, which proves nothing about CG
- probes live in `puzzle/<p>/probe/<lang>/<name>.<ext>`, sent with `agent/tool/oracle.sh <probe> "<question>"` (log `puzzle/<p>/probe/probes.tsv`, committed `chore(golf): probes <p>`); a probe CG confirms at 100% and shorter is copied to `solution/<lang>/main.<ext>` and committed as the solution
- `agent/tool/submit.sh --probe <probe>` (what oracle.sh calls) skips the local gate and the length check, never marks the referee suspect, prints CG progress + our row, exit 3 when CG did not confirm 100% at that length; cap per task in `agent/tool/config.sh` `probe_submits_per_task`
- CG keeps our best submission per language: a lower score stays hidden behind an existing better row. A probe is informative when it is shorter than our row: the row moves (and commits) iff it scores 100%, so each probe is a 1-bit oracle on a hypothesis about all validators
- a language with no row of ours yet shows the exact score of its first submit (% of validators passed): a multi-bit probe, at the cost of a public sub-100% row until a real solution replaces it (allowed, user 2026-10-20)
- CG submit limits are per puzzle (verified 2026-10-20 with `GET /limits?puzzle=<pretty_id>`: 5 / 5 min, 20 / 30 min, 40 / 1 h, 100 / 5 h; test games 30 / 1 min, 60 / 5 min, 100 / 15 min, 300 / 1 h); agents stop at 90 per puzzle per 5 h (`submit_cap_5h`, `submit.sh` refuses past it), the last 10 are the user's
- classic (arcade) puzzles are never played by agents (CG API rule, 403 `arcade-game`): no probes or solution reading there
- test games (`POST /puzzles/<id>/test-games`) only reach IDE tests (validators answer `INVALID_TESTCASE`)
- every fact learned (validator count, value ranges, which input keys separate them, answers) goes in the puzzle's "Hidden validators" section so every language reuses it
- `/progress` score is the best row's score, so a probe's exact score shows only in a language with no row yet (or when it beats the row): spend count probes there, one per language
- validator count from the score grain is UNRELIABLE (refuted blunder-episode-3, 2026-10-24): 12%/14% read as 6/50 and 7/50, the report showed 8 validators weighted 60/70 of 500. Count and weights come from the report; every pre-report count deduction ("25" or "50 validators" in facts.md, e.g. the-fall-episode-2) is suspect until re-read
- gate probe: a known-good general method whose output is replaced by garbage when an input feature differs from the IDE test of the same answer; the score counts validators sharing that feature
- a submit in a language where we have NO row creates a row even when it fails, with the partial score (passed validators x 100 / count): the first probe per language reads k of n, not 1 bit. Verified 2026-10-20 dwarfs ruby `p gets.to_i/3+2` -> row score 40. Validator count shows in other players' partial rows (score steps). Spend fresh languages on count probes (constant answer, candidate key), keep 1-bit probes for languages with a row
- before hunting a hash: probe weaker algorithms from strong to weak (each must pass the IDE-like cases you expect in validators); a pass commits a real gain at once. genome-sequencing: a single input-order fold passed where local validators demand all permutations
- score oracle (verified the-bridge-episode-2, 2026-10-20): `/progress` `score` is the best over ALL languages, so it says nothing about a probe; but a probe in a language where we have NO row creates a leaderboard row carrying the probe's own score (`own row ... "score":49.9`). One such probe measures the share of validators a hypothesis passes (a count, not a bit). Spend it on a hypothesis whose partial count is informative (e.g. hardcoded test answers keyed on input features: share of validators that equal tests). The row then stays at that partial score until a 100% submission replaces it
- histogram probe (verified 2026-10-20 dwarfs): in a fresh language submit the general solution that prints its answer only when a predicate on the input/answer holds (else garbage); the score is the count of validators matching the predicate (`N<=2` -> 60%, `N` in 5..7 -> 0%). Maps the hidden inputs feature by feature; give each predicate a known nonzero part when a 0% result would be ambiguous with a crash
- monotone chain (verified 2026-10-20, the-resistance validator task, 20 submits): CG keeps the best row per language by score, then length. Probes whose passing set only grows (predicates nested: H<2, then H<3, ...) and whose length drops by 1 each step always become the new row, so the row shows each probe's exact pass count. Lowest-score first, pad the first probe so later ones can shrink. Details in `puzzle/the-resistance/referee/hidden/facts.md`
- scores can be weighted (verified blunder-episode-1, 2026-10-20): 12 validators but probe scores 8.5, 17.0, 41.9, 58.1, 66.4, 74.9 fit no k/n with n <= 398; CG gives each validator its own weight (forum "LOOP (85 pts)"). Before deducing a count from the score grain, check that every observed score fits the same k/n; otherwise read scores as sums of weights
- near-uniform weights hide in rounding (verified the-bridge-episode-2, 2026-10-21): 12 validators at 8.3 x 8 (validators 1..8) + 8.4 x 4 (9..12); 74.8 vs 75.1 (both "9 of 12") exposed it. Each score then reads as (lights passed, heavies passed): a free second number per probe, and it tells which index range a validator is in. Check the two 9-of-12 or 11-of-12 sums before assuming 100/n
- complement gate chain (verified the-bridge-episode-2): answer garbage iff the input matches a key value, so the score is 100 minus the matching validators; scores stay high (91.x), so the probe updates a row already at 74.8 and each next probe (equal score, 10 B shorter) still shows. A key with no match gives 100% and ends the chain (commits a real row): put likely-zero keys last

Transcript probe (verified 2026-10-22, the-fall-episode-3 ocaml 842, YES): in an interactive puzzle, a guessed validator (e.g. a test variant) is confirmed in one probe by running the deterministic general solver locally on the guess, taking a checksum of every turn input up to the last turn (perl `$s.="@a,"` per turn, `unpack"%32C*",$s`), and exiting on CG when the keyed validator's checksum at that turn differs. Checks grid, start and the whole spawn schedule at once for ~60 B. False yes only if the real game ends before that turn.

Order trick (verified same task): a fresh language's first probe reads a count; once the row is a 100% probe, more probes stay readable while each is shorter than the row, so put the long probes first and the general solution last.

Free re-read (verified blunder-episode-3, 2026-10-24): reports of OLD probe submissions (ids in `agent/state/events.log`, `probe submit ... submission=<id>`) are readable with `cg-report.sh <id>` at no submit cost. A gate probe sent before the report existed (score only) now tells which validators passed it: re-read every past probe of the puzzle before planning new ones. Bit probes (correct iff bit j of a feature, a different feature per validator class allowed) then recover a 10-bit value per validator in 10 submits in parallel: blunder-episode-3 (N, first n, first t) of all 8 validators in 16 submits.

Hidden-state predicate in interactive puzzles (verified 2026-10-24, shadows-of-the-knight-episode-1 b17-b20): to ask about a value the game never prints (the bomb), run the honest solver, track the exact set of states consistent with the replies so far, and print garbage as soon as that set holds no state satisfying the predicate. Costs no extra turns, so tight-turn validators still pass. Separate validators inside one probe by a known key (`W`) and give each its own predicate. Read reports carefully: list each validator's pass/fail before deducing (a misread fail cost 10 bits there).

## Hash-bit probes (verified on temperature 2026-10-24)

When a validator's input is pinned to a countable universe (known N, line length, answer, positions) but not to one value: probe J prints the right answer iff bit J of `md5(<canonical input string>)` is 1. One probe reads one hash bit of every validator at once; offline, enumerate each universe and keep the inputs whose bits all match (log2(universe)+4 probes for a safe unique hit). Already known validators double as a pipeline check. Temperature: 5 validators recovered with 20 probes (universes 3k..150k). Weak for unpatterned large inputs (10 free values): there it only screens hypothesis families. Source: `puzzle/temperature/referee/hidden/facts.md`.

## Partial-row language = exact first score (verified roller-coaster 2026-10-25)

A language whose own row is below 100% (from an earlier failed probe) is a *multi-bit* channel for
its first new probe: CG keeps the best submission by score then length, so a new probe scoring more
than the stale partial row replaces it even when it is far longer. The own row then shows the probe's
exact score (sum of validator weights passed), not just a bit.

- pad the first probe to a large arbitrary length (e.g. 5000), read the score, then use lengths
  4999, 4998, ... for later probes: every probe replaces while its score >= the row's score (equal
  score needs a shorter length), so a bit=1/bit=0 probe is always readable from score or length.
- `submit.sh` breaks its own-row wait early when the row length is already <= probe length (fresh
  partial row); the POST is still sent, so poll `leaderboard_rows <puzzle> LANGUAGE <lang>` manually.
- pick a weight layout where the target validator's failure is a unique score: roller-coaster
  1150 total, 3x150 heavies -> target fails = 1000 (86.9565%), all-but-V8-and-heavies = 850 (73.913%).
- the language must still execute the probe on the heavy validators: roller-coaster used
  `spawnShell.wait` in a precompiled language (D) with a fast perl method A core (~0.03 s). A slow
  method C core timed out; the timeout score was 850, distinguishable.

## progress criteria_score = fast row oracle (verified 2026-10-25, the-resistance ruby)

`GET /puzzles/<pretty_id>/progress` returns `criteria_score` = the code size of our *best*
submission; it drops to a probe's length within ~20-40 s of a 100 % probe, while our leaderboard
row can lag ~4 min (and `submit.sh --probe`'s miss path polls it for 210 s). Fast loop: POST with
`submit.sh --no-wait --probe` (set `CG_PROBE_QUESTION`), poll progress every 6 s, stop when
`criteria_score` changes; resolve the `pending, submission <id>` row of `probes.tsv` by hand with
the same awk `submit.sh` uses. A miss leaves `criteria_score` unchanged. Do not run several probes
in parallel: a hit from one makes the others' row check pass against the wrong probe (they can
then be copied into the solution file).

## Row oracle needs a *shorter* 100 % probe: no isolation without the report (2026-10-25, blunder-episode-3 python)

The per-validator report is the only multi-bit channel. When `cg-report.sh` says "no CG session",
the fallback row oracle only fires when the probe scores 100 % **and is shorter than our own row**
(CG keeps the best score then shortest length per language). A per-validator isolation probe must key
on the input to know each validator's answer and carry a predicate, so it is always longer than the
shortest hardcode row of that language: even at 100 % it does not move the row and leaves no trace
(and `/progress` keeps the global best, often another language, so `criteria_score` does not move
either). Consequence: without the report, each submit is a 1-bit test of a *complete candidate
solution* shorter than the row; a k-bit unknown cannot be bisected. Plan exploits accordingly (test
the top-k candidate tables, or fix the session first), do not spend probes on long isolation code.
