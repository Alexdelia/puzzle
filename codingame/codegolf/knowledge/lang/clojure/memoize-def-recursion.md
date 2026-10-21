# Memoized recursion: def inside -main

Verified the-fall-episode-3 2026-10-21 (local referee 11/11, CG 100% at 947).

- `(def f(memoize(fn[g p r]...(f g n m)...)))` inside `-main` after a `let`: the fn closes over the let locals, the recursive call goes through the var, so every level hits the memo. Args may be vectors and lazy seqs (hashed by value; hashing realizes a lazy seq of `(read)`s in order, so no `doall` needed before the call)
- a `(fn f[...])` self-name bypasses the memo; `letfn` has no memo
- Clojure 1.11 has `abs`: `(abs x)` beats `(Math/abs x)` by 5
- trap: `(not= a b c)` is true unless all three are equal; "c differs from a and b" = `(not(#{a b}c))` or `(not-any? #{c}[a b])`
- `(some(fn[[g s :as z]]...(and ok z))cands)` evaluates the predicate one element at a time; `filter`/`for` over a vector run in 32-chunks (no early exit)
- local interactive referee counts JVM startup (~1 s, Java 8) in the first-turn limit; CG does not (the-fall-episode-3: 1000 ms first turn passed on CG, timed out locally)
