# Recursion with a subshell function

Verified: genome-sequencing bash (local referee 5.1.16 + CG accepted, 2026-10-20).

- `f()(...)` (parens body) runs every call in a subshell: all variables (loop vars, temporaries) are local for free, no `local`, recursion never clobbers the caller's `for` variable
- results go to stdout: leaves `echo` a value, caller pipes `f|sort -n|sed q` for the min (`sed q` 2 bytes shorter than `head -1`)
- a child inherits every variable of its caller: reset a per-node flag with a prefix assignment on the call, `k= f $s` (temp env reaches the function)
- leaf flag for free: a loop variable that only runs in non-leaf nodes (here the inner `for k in {0..9}`) ends at its last value; `echo $k${#1}` prints `9<len>` in inner nodes (large, never the min) and `<len>` in leaves
- cost: one fork per node, about 0.1-0.3 ms locally; ~15k nodes took 4 s, mind the 6 s io limit
- `${t:{0..9}}` does not brace-expand (syntax error): brace expansion is not done inside `${}`
