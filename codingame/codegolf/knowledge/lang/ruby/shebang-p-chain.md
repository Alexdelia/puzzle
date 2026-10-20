# `#!ruby -lp` + Kernel gsub chain (verified chuck-norris, 14/14)

Ruby honors switches on the `#!ruby` line. With -p/-l, Kernel `gsub` works on `$_` and returns it, so
`gsub(/./){"%07b"%$&.ord}.gsub!(/(0)+|1+/){"0#$1 #$& "}.chop!.tr!"1",?0` chains in place (no `$_.`). Unmatched group in block interpolates empty. 81 bytes total.
