# The Fall ep.3 general DFS in Haskell (1086 B, 11/11 local, CG submitted rank #3)

- flat list grid, leading 0 per row (stride w+1) + padding row; cell index k=y*v+x+1, state p=k*3+side; exit table as 42-char digit string; list `!!` is fast enough (turn << 150 ms)
- stdout needs `import System.IO` + `hSetBuffering stdout NoBuffering` (block-buffered on pipe, turn-1 timeout otherwise)
- candidates as list comprehension, first plan via `q g p r` list head; WAIT when empty
- no hardcode possible without hidden validators; see knowledge/puzzle/the-fall-episode-3.md
