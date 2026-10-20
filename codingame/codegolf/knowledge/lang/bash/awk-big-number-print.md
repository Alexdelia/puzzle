# awk prints big integers in exponent form

Verified locally (mawk 1.3.4, referee validator 12, answer 8589934592): `print s` with `s` >= 2^31 gives `8.58993e+09`. Fix: `printf"%.0f",s` (13 bytes, exact below 2^53) or `-vOFMT=%.0f` with `print` (18 bytes). `printf"%d"` is not safe: mawk 1.3.3 (CG) clamps `%d` to 32 bits (unverified on CG, known mawk behaviour).

Perl avoids it: `say` of an IV prints all digits.

# Header count as sign weight (awk after `sort -nk2`)

`read n;sort -nk2|awk -vn=$n 'NR>n/2{s+=$2}NR<n/2+1{s-=$2}...'`: top half minus bottom half of sorted y, middle of odd n counted in both so it cancels; no array and no END loop. 113 bytes whole cell with x min/max patterns `NR<2{a=b=$1}$1<a{a=$1}$1>b{b=$1}`; the array + END loop form is 121.
