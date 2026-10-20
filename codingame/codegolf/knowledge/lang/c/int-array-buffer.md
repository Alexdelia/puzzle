# int array as char buffer

Verified ascii-art local 13/13 (2026-10-20). `t[99];` (implicit int) is a valid `scanf("%s")`/`gets` target (warning only) and shorter than `char t[999]`. Pointer arithmetic on it scales by 4: buffers you offset (`r+l*k`) must stay `char`. Declare `t[99];char*p,r[999];`. `main(l,i)` params as free int vars (`&l` fine). Compare `i>25u` is unsigned, replaces `i<0|i>25`.
