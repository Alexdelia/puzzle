# Loop `c` times as a fold over `make c 0`

Verified locally and on CG (roller-coaster row 257, 2026-10-22, c = 1e7: 82 MB peak, 0.2 s).

- `fold_left(fun v _->STEP)INIT(make c 0)` (inside `Array.(...)`) iterates a pure state `c` times: 11 bytes shorter than `let v=ref INIT in for _=1 to c do v:=STEP done;... !v` and 7 shorter than `let rec r c v=if c<1 then v else r(c-1)(STEP)in r c INIT`
- cost: an 8-byte-per-element array, `c` up to ~5e7 stays under CG's ~768 MiB cap (`knowledge/lang/_shared/memory-limit.md`)
- pair state packed into one int (`v=a*n+i`, step `v+d.(v mod n)` with `d` a delta table) avoids tuples in the fold
