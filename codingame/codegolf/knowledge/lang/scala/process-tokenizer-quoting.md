# Scala: quoting rules of `sys.process.Process(String)` and the Scala box interpreters

Source: scala 2.13.5 `Parser.scala` (https://raw.githubusercontent.com/scala/scala/v2.13.5/src/library/scala/sys/process/Parser.scala), behaviour checked locally on generated wrappers 2026-10-22.

- `Process("cmd args")` splits on Java whitespace (newline included); `'..'` and `"..."` group, quotes removed, adjacent pieces join like in sh (`a' 'b` = one word `a b`)
- outside quotes a backslash is a plain character (no escaping, kept); inside quotes `\` only stops the next char from closing the quote and is kept too: `'\''` is an unmatched quote, so a literal quote char goes inside the other quote kind (`"'"`, `'"'`)
- unmatched quote: `Parser$ParseException: Unmatched quote` at run time, not at compile time
- cheapest encoding of a core: quote only whitespace and quote runs (+2 per run), attach the core to the flag word (`perl -anE<core>`, `python3 -c<core>`, `lua -e<core>`, `php -r<core>` all accepted); not for `sh -c` (dash parses `-c<core>` as flags)
- triple-quoted `"""..."""` still applies `\uXXXX` unicode escapes in 2.13.5 (`"""A"""` prints `A`, local scalac): a core containing `\u` must use a plain string with `\\`
- Scala box PATH (CG test games 2026-10-22, temperature-code-golf test 1): `sh` (dash), `perl`, `python3`, `lua`, `php` run; `ruby` and `node` give `java.io.IOException: Cannot run program "ruby": error=2`
- generators: `agent/wrap/scala/` (Process with plain / triple-quoted literal, attached flag, and `import sys.process._;Seq(..).!<`)
