# Haskell: flood fill with Data.IntSet

Verified surface 17/17 local, CG submitted 242 (rank 4):
- `import Data.IntSet` unqualified: `map`/`filter` become ambiguous, use `fmap`; `member`, `delete`, `fromList` fine
- set of water indices from `[i|(i,'O')<-zip[0..](unlines r)]` (stride L+1 via unlines, walls = missing members, no bounds check)
- flood: `f s(i:q)|member i s=1+f(delete i s)(i-1:i+1:i-w:i+w:q)|1>0=f s q;f _ _=0`, persistent set so each query restarts from the full set; 360k lake ok in ~1 s
- `[x,y]<-fmap words r` in a comprehension skips rows and the N line; `fmap(fmap read.words)` instead of two `read` is a tie (242)
