# Console.in.lines instead of io.Source

- `io.Source.stdin` fails on CG: `object Source is not a member of package io` (some other `io` shadows `scala.io`); local referee accepts it, so a 0% row hides this. `scala.io.Source.stdin.getLines` works (verified, test game).
- `Console.in.lines` (java Stream[String] from the BufferedReader) is 15 shorter than `scala.io.Source.stdin.getLines`: `Console.in.lines.skip(1).reduce((s,t)=>...).get` compiles on 2.13.5 with untyped lambda params (Stream.reduce(BinaryOperator) resolved by SAM), CG 100% on genome-sequencing (verified).
- `reduce` as fold with no initial value: first line becomes the accumulator, no `var`.
- `t.tails.filter(p).toSeq.last` = shortest suffix satisfying p, 9 shorter than `(0 to 7).map(t.takeRight).find(p).get`.
