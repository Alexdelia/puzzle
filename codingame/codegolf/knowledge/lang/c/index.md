# C

- [env](env.md): gcc 11.2.0 gnu17 -O0, K&R implicit int/decls and `gets` accepted, PIE without ASLR, verified
- [int-array-buffer](int-array-buffer.md): implicit-int arrays as gets/scanf targets, char only where offset, `i>25u` unsigned clamp, verified
- [strstr-glyph-lookup](strstr-glyph-lookup.md): transposed font table + strstr index + one write per glyph, verified
- [system-perl](system-perl.md): `system("perl -anE'...'")` wrapper 29 bytes, flush via `$|+=guard`, set-if-unset hash from line slice, verified on dont-panic
