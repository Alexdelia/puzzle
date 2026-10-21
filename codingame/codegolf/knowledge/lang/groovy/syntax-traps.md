# Groovy syntax traps (verified local 3.0.8, 2026-10-21)

- no `.9` literal: write `0.9`
- `String[]` index past the end throws, so `arr[i]?:x` is no cap; use `Math.min`
- `print"..."` without space fails to parse with a GString containing nested quotes; `print 'a'+x+'b'` is safe
