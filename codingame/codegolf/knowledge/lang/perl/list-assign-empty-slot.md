# List assignment: an empty slot is dropped, it does not skip the RHS

`($a,$b,,$y)=@F` does **not** assign `@F[0],@F[1],undef,@F[2]`: the empty slot is
removed from the LHS list, so `$y` gets `@F[2]` (Perl 5.34, CG box).

```
($a,$b,,$y)=(1,2,3,4)   # 1 2 3
($a,$b,undef,$y)=(1,2,3,4)  # 1 2 4
```

To skip one field of an autosplit header, either use a filler variable
(`($a,$b,$x,$y)=@F`) or a slice (`($a,$b,$y)=@F[0,1,3]`). Filler (14 chars) is
3 under the slice form (17).

Verified locally with CG perl 5.34; caused a wrong 61-byte draft to skip
thorY and use thorX instead (`y=7` instead of 8).
