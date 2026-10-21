# os.execute with [[ ]]

Verified 2026-10-21 on the-resistance (local 18/18, CG 100% at 209, rank #4): `os.execute[[perl -nle'...']]` wraps a perl one-liner containing `"` and `'`-free-shell-safe code at 13 bytes of wrapper; beats native Lua DP (374) by 165. Use `[[ ]]` when the core has `"`, `"..."` otherwise (see os-execute-wrapper.md).
