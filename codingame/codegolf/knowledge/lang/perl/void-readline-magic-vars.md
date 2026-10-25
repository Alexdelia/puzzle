# `<>` in void context does not set `$_`; `<>=~/.../` reads and matches in 2+pattern bytes

In plain file-scope code (no `-n`/`-p`), the statement `<>;` reads a line but **does not assign it
to `$_`** (verified Perl 5.34: `<>; print "[$_]"` prints `[]`). `while(<>){...}` and `-n`/`-p` do
set `$_` because the loop condition evaluates `<>` in scalar context.

To read one line and run a regex on it without the 3 B `$_=<>;`, use the match on the read value:

```
<>=~/ .* /     # 10 B: reads one line (scalar context) and matches it, no space needed before =~
```

`$&`, `` $` ``, `$'` are then set from that match, which is all a one-shot needs:

```perl
<>=~/ .* /;$h=(W,E,N)[$`%3];...      # lightX in $`, lightY-mid in $&, thorY in $'
```

Verified 2026-10-25 golfing power-of-thor (one-shot plan, local referee hidden 4/4).

- `<>=~ /x/` (with spaces) is the same length as `<>=~/x/`; both parse, the space is optional
- a failed match on later input lines does not reset the magic vars in plain code (only `-p` does),
  so the same 10 B read works for a per-turn loop that only needs the init line once
- `$_=<>;/ .* /` is 13 B, `for(<>){...}` slurps stdin in list context (blocks interactively)
- `<> =~ / .* /` where the pattern needs a *trailing* space to leave the last field in `$'`:
  `/ .* /` is greedy from the first to the last space, so `$&` starts with `lightY` and `$'` is
  `thorY`; shorter patterns (`/ .*/`, `/.+ /`) lose one of the two fields
