# Scala interactive argmax (the-descent)

`object Player extends App{while(1>0)println((0 to 15).maxBy(_=>System.in.read)>>1)}` 83, CG 100% (2026-10-21). maxBy calls key once per element in order; newline(10) < digit so index>>1. `readLine` bare does not exist, `io.StdIn` passes locally but fails on CG. Local referee only launches `Solution`: test with a renamed copy, then send via oracle.sh probe (submit.sh refuses `Player`). Top 70 not reached.
