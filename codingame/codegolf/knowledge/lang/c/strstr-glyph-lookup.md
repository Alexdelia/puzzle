# glyph lookup by strstr on a transposed table

Verified mayan-calculation local 21/21 (2026-10-20). Size 373 vs 404 for per-glyph hash table + printf.

- store each font glyph contiguous with its row newlines: `G[i/L*B+j*-~L+i%L]=f[i],G[s+1]=10` (`B=-~L*H`, `s` the index); static zeros make `G` a C string
- read the query glyph raw with `for(i=0;i<B;)r[i++]=getchar()` (newlines included) into a buffer whose tail beyond `B` is zero (a separate buffer for font rows: stale tail breaks strstr)
- index = `(strstr(G,r)-G)/B`; declare `char*strstr();` in the same declaration as the arrays (implicit int return would truncate PIE pointers)
- print the found glyph with one `write(1,G+d*B,B)`, newlines already inside
- misaligned matches did not occur in the .o_ / digit / .X fonts
