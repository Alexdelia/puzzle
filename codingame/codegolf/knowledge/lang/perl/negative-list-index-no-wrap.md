# Perl list indices beyond -@a are undef (no wrap); only -@a..-1 count from the end

Contrast with Python/Ruby, where any negative index wraps modulo the length. In Perl:

```perl
@a=(1,2,3); print $a[-4]   # undef (with "Use of uninitialized value" warning), NOT $a[2]
```

Verified Perl 5.34, 2026-10-25 (power-of-thor golf: `(W,N,E)[-$`]` looked like a 1 B cheaper
letter lookup than `(W,E,N)[$`%3]`, but `$a[-28]`, `$a[-5]`, `$a[-31]` are all undef, not wrapped).

Consequences:
- a modulus on the index is mandatory when the index can exceed the array: `[$`%3]`, not `[-$`]`
- do not assume the Perl/Ruby negative-index idiom (`("",S,N)[$v]` for v in -1..1) generalises
  beyond -@a: only -1, -2, -3 are safe on a 3-list
- a scalar `%` with an expression on the right still needs the value; the 1 B saving has to come
  from somewhere else (in power-of-thor nothing did: 13 B list-index is the floor found)
