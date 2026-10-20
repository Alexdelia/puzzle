# Endless loop as `. $0`

Verified on the-descent (local referee and CG, bash): ending the script with `;. $0` re-sources the script file, endless loop for 5 bytes (`f()(BODY;f);f` costs 9, `for((;;)){ BODY;}` 13). Works on CG: `$0` is a path bash can open. Depth is bounded by the game length (about 9 turns here); each source nests one level, no fork.

Stdin stays shared, so a per-turn `sed 8q` reads one turn and leaves the rest.
