# read assigns names left to right

Verified locally (bash 5.1.16) and on CG (dont-panic submission accepted 2026-10-20).

- `read f e[f] r` stores field 2 into `e[<field 1>]`: each name is bound as its field is split, so a subscript sees variables assigned earlier in the same `read`
- the last name takes the rest of the line: add a throwaway last name (`read a a a f e[f] a`) when the array element must hold one number, or arithmetic on it fails with a syntax error
- a throwaway name can be repeated and reused right after (`read a a a ...;a=(X Y)`)
