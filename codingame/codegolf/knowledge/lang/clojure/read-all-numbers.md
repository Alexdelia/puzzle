# Read all numbers of stdin

`(read-string(str\[(slurp *in*)\]))` gives a vector of all numbers (verified blunder-episode-3, CG). Output through `println`: `printf`/`print` output is lost (no flush at exit with the gen-class main, verified locally). Float literals need the leading zero (`0.9`, not `.9`).
