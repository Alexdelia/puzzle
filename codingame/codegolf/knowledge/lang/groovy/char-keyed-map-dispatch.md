# Groovy: chars as direction keys, closure-map dispatch, slashy-string lookup

Verified on blunder-episode-1 (374 bytes, CG rank 5, 24/24 local):

- direction kept as the letter itself: `D=[S:w,N:-w,E:1,W:-1]`; modifier cell becomes `k=D[c]?c:k` (instead of `[0,1,2,3,k]['SNEW'.indexOf(c)]`, -20 bytes); priority list a String `P='SENW'`, `(k+P).find{...}` iterates its chars, `P.reverse()` works on String
- per-cell effects: `[B:{b^=1},I:{P=P.reverse()},X:{m[p]=' '},T:{...}][c=m[p]]?.call()` beats a chain of `if(c=='B')` (-11); closures assign script-binding vars fine
- map literal can define a var inline: `D=[S:w=expr,N:-w,...]`
- name lookup by regex over a multi-line slashy string with literal newlines: `/SOUTH<NL>EAST<NL>NORTH<NL>WEST<NL>/.find(/$k.+\n/)` returns name + newline
- cap loop: `while(m[p]!='$'&n++<999){...}` then `print n<999?o:'LOOP'`
- `s.split()[2].size()+1` = row width when row 1 is all `#`
