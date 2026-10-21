# os.execute with [[ ]]

Verified 2026-10-21 on the-resistance (local 18/18, CG 100% at 209, rank #4): `os.execute[[perl -nle'...']]` wraps a perl one-liner containing `"` and `'`-free-shell-safe code at 13 bytes of wrapper; beats native Lua DP (374) by 165. Use `[[ ]]` when the core has `"`, `"..."` otherwise (see os-execute-wrapper.md).

chuck-norris (2026-10-21): `os.execute[[perl -pe'<core>']]` = 14 + bash-cell length; 81 with the 67-byte bash core vs native Lua gsub two-pass 146. Lua is always bash cell + 14, so the Lua top 78 implies a 64-byte bash/perl core. `q(...)` instead of `"..."` in core to use the `"` Lua delimiter ties (saves 2, costs 2).
