# System.Cmd + bash -c wrapper (shadows-of-the-knight-episode-2, CG 259 #1)

- `import System.Cmd;main=system"bash -c '<core>'"`: `System.Cmd` imports on GHC 8.4.3 (4 bytes shorter than System.Process), `system` is /bin/sh = dash so bash features need `bash -c ` WITH the space (`-c'..'` is parsed as flags). Overhead 31 + core
- core must have no `"` or `\`; newlines become `;`; backticks fine inside the single quotes
- bash arithmetic: ternary else-branch cannot be an assignment (`a?b:c=1` errors), wrap `(c=1)`; middle branch may assign; `(t+S)/2` -> `t+S>>1` inside a ternary middle
