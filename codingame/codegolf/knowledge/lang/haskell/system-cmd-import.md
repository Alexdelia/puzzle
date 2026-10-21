# System.Cmd instead of System.Process

`import System.Cmd;main=system"..."` works on GHC 8.4.3 (local referee and CG): 4 bytes shorter than `import System.Process`. Verified the-descent: `import System.Cmd;main=system"sed 8q|nl -v0 -w1|rev|sort|tail -c2">>main` 72 bytes, CG #1 (top was 74). `>>main` loops the shell call (6 bytes, no flush trouble since sed process writes straight to the pipe).
