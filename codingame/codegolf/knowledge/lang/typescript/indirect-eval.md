# `eval?.("js")`: indirect eval, fast hot loops

Verified local 2026-10-21 (roller-coaster, CG accepted 202 then shorter): direct `eval("...")` runs inside the commonjs module wrapper, so every implicit global is looked up dynamically: a 1e7-iteration memo loop timed out (10/16) where the same code with `//@ts-ignore` passed 16/16. `eval?.("...")` (optional call) is an indirect eval per spec: code runs in global scope, full speed, tsc accepts it, 10 B overhead (2 more than `eval(`, 3 less than ts-ignore). Use it whenever the body has a heavy loop; plain `eval(` is only safe for light bodies.
