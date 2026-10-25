# `-F` with no pattern: `-a` splits a line into characters

`perl -F -alE'...'` (`-F` empty, `-a`, `-l`, `-E`): `-a` fills `@F` with the line's
characters, not words. Empty `-F` is spelled `-F` followed by the next switch
(`perl -F -alE'` and `perl -aF -lE'` are both 13 B, `perl -aFlE'` would take `lE` as the
pattern and is wrong).

- `-a` implies `-n` (no `-n` needed)
- `-l` chomps before the split, so there is **no trailing empty field**:
  `printf 'abc\n' | perl -F -alnE'say scalar@F'` -> 3. Without `-l` it is 4
  (trailing empty maps to `-1`, an extra `?` glyph here)
- mapping characters without a capture regex: `map ord()%32*/\w/-1,@F` parses without
  parens (letters 0..25, every non-word char `-1`; `''` would also be `-1`, hence `-l`)

Measured on ascii-art hidden (7/7 pass, `run.sh --cg`):

    perl -F -alE'$l||=$_;$.<4?@t=map ord()%32*/\w/-1,@F:say+(/.{$l}/g)[@t]'
    # core 57 B vs 59 B for map-over-$_ with /(\w)|./g; command costs +3 B

Net +1 B in a Python `os.system` wrapper (94 vs 93), so it loses there; it wins only
where the split is already active or flags are free (e.g. a `-a` shebang core).
