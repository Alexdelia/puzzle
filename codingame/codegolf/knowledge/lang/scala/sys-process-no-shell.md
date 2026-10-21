# Scala: `sys.process.Process("cmd").!<` runs a core without a shell

Verified 2026-10-21 (dont-panic, CG accepted 135, interactive): `object Player extends App{sys.process.Process("perl -anE<code>").!<}` = core + 52 bytes, stdin connected, interactive turns answered (perl flushes itself).

- `Process(String)` splits on whitespace (quotes group words), no shell: `$ ; & | ( ) { }` need no escaping, and a core without spaces needs no quotes (`perl -anE%e=...`)
- `!<` connects stdin, `!` does not (0%, see `python-wrapper-interactive.md`); postfix `...)!<}` is a compile error, write `.!<`
- `import sys.process._;"cmd".!<` is the same length
- 48 bytes shorter than the ProcessBuilder form of `python-wrapper-interactive.md` for a one-word-per-arg command
