# Linear RLE decode: Array.fill with alternating colour, program in the handler

Verified locally and on CG (music-scores OCaml 379, 100%, 2026-10-22).

- `let rec r s=try Scanf.scanf" %c %d"(fun c n->r(s^String.make n c))with _->...` is quadratic: times out (>6 s) on 1M pixels / 12k runs
- linear and short: `let a=Array.make(w*h)0 in let rec r k v=Scanf.scanf" %_c %d"(fun n->Array.fill a k n v;r(k+n)(1-v))in try r 0 0 with _->MAIN`: colours alternate, so `%_c` skips the letter and `v` toggles (first run must be the 0 colour); EOF raises inside `Scanf`, the handler runs the rest of the program once (only the innermost `try` catches; an exception raised in MAIN propagates to the outer frames and reruns MAIN on a shorter prefix, so MAIN must not raise)
- ints 0/1 make the pixel directly usable as an index: `"HQ".[a.(i)]`
