# `/progress` score is the best row, not the latest submission

`GET /api/v1/puzzles/<pretty_id>/progress` mixes fields from two sources: `submission_id` and
`in_progress` are the latest submission, but `score` and `criteria_score` are the user's **best
row** (best score and its length). A probe or a failed submission therefore never shows its own
partial score there.

Verified 2026-10-25 (power-of-thor ruby): submitted a guaranteed-0% 42 B probe (`puts ?Z` + pad,
submission 41563396); the next progress read said `submission_id: 41563396, in_progress: false,
score: 100.0, criteria_score: 45.0` (the 45 B 100% row). Same reading after two 42 B failed probes.

Consequences:
- the only probe oracle on CG is the **leaderboard row**: a 100% probe shorter than our row moves
  it (usually within ~1 min, resubmit once if it does not); a failed probe leaves the row alone.
- `submit.sh` polls the row up to 210 s on a failed probe, which is the cost of a failed probe
  (~4 min), not of a successful one.
- `cg-report.sh <submission-id>` (per-validator pass/fail) needs a CG web session; without one,
  partial validator results of a probe are unobservable. Design probes assuming a row-move-only
  oracle.
