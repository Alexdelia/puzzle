# `dd` is a 1-byte shorter stdin passthrough than `cat`

Verified on CG 2026-10-25 in Ruby (dwarfs-standing-on-the-shoulders-of-giants
ruby 30 B, 100 %, row moved 31 -> 30, probe `probe/ruby/dd30.rb`):
`` `dd` `` (4 B) returns the whole stdin exactly like `` `cat` `` (5 B), so a
backtick read costs 1 B less. GNU `dd` copies stdin to stdout by default and
writes its "records in/out" summary to stderr, so captured stdout is the raw
input. Tested with the real CG sandbox (`dd` is on PATH there).

- e.g. `` p`dd`.size `` prints the input length; `t=`dd`;p ...` replaces
  `` t=`cat`;p ... `` for free.
- not verified in other languages' sandboxes, but `dd` is coreutils and was
  present for Ruby on CG. No other 2-char passthrough was found: `sh` executes
  stdin as a script, `tee` needs a file operand, `bc`/`ed` parse stdin, `rev`
  reverses.
