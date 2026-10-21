# Left section shares its operand

`(e-)` desugars to `(-) e` in GHC: `e` is evaluated once per mapping even at -O0. So `sum(map(abs.(sort(y++y)!!length y-))y)` is O(n log n) without a `let` (network-cabling test 04 N=100000: 1.3 s at -O0, 8 bytes shorter than `let`/comprehension). Verified locally.

Median of unsorted y without length arithmetic: `sort(y++y)!!length y` (upper median for both parities).
`transpose.map(map read.words).tail.lines` splits `x y` rows into columns with only `import Data.List`.
