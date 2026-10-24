# `puts` supplies the last newline; literal LF in literals; `$&` in scan blocks

- `puts(expr)` appends `$\` ("\n") unless `expr` already ends with "\n", so an answer stream built as `"WAIT\n"*n+"x y"` needs no trailing `\n` in source: saves 1 B per literal. Verified vox-codei-episode-2 ruby 153 (CG 100%)
- a raw LF inside a double-quoted literal (`"WAIT<LF>"`) is 1 source byte, same as the source's own newline; `"\n"` is 2. Count = UTF-8 bytes, so this is free. Same for the final literal that `puts` terminates
- inside `String#scan(/../){...}` `$&` is the matched pair: `{a,b=$&.bytes;...}` is 2 B shorter than `{|q|a,b=q.bytes;...}`, and bytes are ints (no `.ord`)
- `a+'AB'` before a method call needs parens: `(...+'AB').scan`; `.scan` binds to `'AB'` otherwise (+2 B). Cheaper trailer than printing 3 WAIT lines (13 B)
