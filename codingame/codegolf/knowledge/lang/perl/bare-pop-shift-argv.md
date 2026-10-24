# bare pop/shift at file scope use @ARGV, not @_

`pop` / `shift` without an argument pop `@ARGV` at file scope even when `@_` is non-empty
(`@_=(0,2,4);for(@_){pop}` leaves `@_` length 3 and returns empty). Inside a real `sub{}` they use
`@_`. Consequence for one-liners: `@_=sort{...}LIST;...pop` does NOT pop that array, and a bare
`pop`/`shift` cannot replace `pop@x`/`shift@x` in a `-p`/`-0p` file body. Wrapping the body in
`sub{...}()` costs 7 B (4 + 3) and saves only 3 B per bare use, so it rarely pays.
Related parse traps: `pop-shift`, `pop-$_[0]`, `$s+pop-$_[0]` are "Experimental pop on scalar"
errors; `pop(sort{...}@a)` is a compile error too; use `pop@a`, `$a[0]`, `(sort{...}@a)[-1]`.
Verified locally on perl 5.42 (behaviour is long-standing); CG perl is 5.34.
