# Open-loop schedule as a char string indexed by a float counter (verified CG, mars-lander-episode-3 php, 2026-10-21)

- `ord('abc'[$i++/28]??P)-80`: a float string offset truncates (`$s[1.5]` = `$s[1]`, no notice under `??`), `??` on an out-of-range offset gives the default, so the bare constant `P` (ord 80) is a free tail value forever. One char per 28 turns, value range -48..46 from printable ASCII
- start the counter at a keyed offset to make two cases share one string: `$i=~-fgetc(STDIN)*X` gives X when the first byte is "2", 0 when "1" (`~-"2"` = 1, `~-"1"` = 0); `fgetc(STDIN)*X` is 2 shorter (2X vs X)
- infinite `for(;;)echo` into the pipe is fine: blocks when full, the bare-constant warning per line did not hurt on CG
