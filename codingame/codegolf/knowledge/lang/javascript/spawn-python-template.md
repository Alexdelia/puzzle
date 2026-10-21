# Run a Python solution from JS: spawnSync + template literal

`require('child_process').spawnSync('python3',['-c',`CODE`],{stdio:'inherit'})`: 71 B overhead, CODE pasted raw (real newlines, no `\n` escapes) as long as it holds no backtick, backslash or `${`. Interactive stdin/stdout work (inherit).

- vs `writeFileSync('/tmp/p.py',"..\n..")+execSync('python3 /tmp/p.py',{stdio:'inherit'})`: saves the file write and one byte per newline. Mars Lander 3: 2681 -> 2026 (gate also removed), CG 100%, verified 2026-10-21
- CG JS box has `/usr/bin/python3` 3.11.5 (knowledge/puzzle/mars-lander-episode-3.md)
- node + python startup eats first-turn time: under heavy local load the referee times out turn 1 (1000 ms), retry when the machine is idle
