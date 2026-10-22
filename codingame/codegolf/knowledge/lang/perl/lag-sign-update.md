# Lagged sign update: track a coordinate inside the index expression

Verified on CG power-of-thor (100%, 94 bytes, was 96).

- greedy step toward a target, one axis: `$y+=$v=$b<=>$y;...("",S,N)[$v]` needs a statement and a `;`
- lag form: `("",S,N)[$v=$b<=>($y+=$v)]` applies last turn's step first, then takes the new sign. Same characters, but the update lives in the index, so both axes and the output fit in one statement: `$_=("",S,N)[$v=$b<=>($y+=$v)].("",E,W)[$h=$a<=>($x+=$h)]`, 2 `;` saved
- `$v` starts undef (adds 0), no init; parens needed (`<=>` binds tighter than `+=`)
- same idea in any language with assignment expressions and a 3-way compare (Ruby `<=>`, PHP `<=>`); where locals must be initialised (Ruby `nil`) the init may eat the gain
