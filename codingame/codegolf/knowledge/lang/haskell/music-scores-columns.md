# Image columns in Haskell (music-scores, CG 100% at 337, 2026-10-21)

- one chunker `g n[]=[];g n s=take n s:g n(drop n s)` serves both the RLE token pairs (`[c,n]<-g 2 r`) and the image rows (`g w`); `transpose` (Data.List) gives columns, indexing the flat list with `!!` is far too slow on 80k pixels
- decode pixels straight to Bool: `replicate(read n)(c>"B")` (black False): `zipWith(<)col staff` = black where staff white, `"QH"!!fromEnum px`
- first-true row of a column: `length$takeWhile not$..`; "previous column unflagged" via `zip3 m(h:m)(drop 5 k)` with guards, beats collapsing runs with words/take 2 (355)
- `words t` of the whole input gives `w:h:rle`, no lines needed
