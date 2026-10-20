# Perl flood fill in a bash wrapper (surface, 157)

Verified 17/17 local, CG #1.

- `perl -0777nE'...'`: whole stdin in `$_`
- `s/(\d+)\n.*\n/1x($w=$1+1)/e`: replace the 2 header lines by `w` pad chars, stride `w=L+1`, row 0 up-overflow lands in pad
- worklist: `@k=start; ... push@k,... for@k` iterates while growing; `substr($t,$_,1,X)eq O&&push@k,$_-1,$_+1,$_-$w,$_+$w` marks and tests in one 4-arg substr (land overwritten too, harmless)
- answer `~-@k/4` (`@k` = 1+4*size, `~-` = decrement)
- 4-arg substr past string end dies: `$t=$_.$_` copy gives tail pad (3 bytes)
- per-query 1 MB string copy is fast enough (test 8, 100 queries)
