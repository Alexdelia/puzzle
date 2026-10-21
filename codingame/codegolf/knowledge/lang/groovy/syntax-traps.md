# Groovy syntax traps (verified local 3.0.8, 2026-10-21)

- no `.9` literal: write `0.9`
- `String[]` index past the end throws, so `arr[i]?:x` is no cap; use `Math.min`
- `print"..."` without space fails to parse with a GString containing nested quotes; `print 'a'+x+'b'` is safe
- `String[] as long[]` throws `GroovyCastException` on multi-digit strings (`'10'` to `long`): use `*.toLong()` (verified local 2026-10-21, roller-coaster)
- Long `/` Long gives a BigDecimal; an exact quotient prints as a plain integer (`1000L*20/2` prints `10000`), so `x*(c-(c%=q))/q` is a safe floor-multiply (verified local)
