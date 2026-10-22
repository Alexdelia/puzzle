# Free empty hash in the arg unpack

`my($i,@r,%h)=@_;` declares a fresh empty `%h` (the array slurps all args), 4 under `my($i,@r)=@_;my%h;`. A lexical, so it survives recursive calls of the same sub.

Verified on CG: the-fall-episode-3 perl 670 (2026-10-22).
