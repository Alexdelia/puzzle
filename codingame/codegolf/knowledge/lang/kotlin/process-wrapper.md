# Spawn an interpreter from Kotlin

Verified 2026-10-22 (local kotlinc 1.7.10 + CG temperature-code-golf test games, journal `agent/state/journal/wrap.-.kotlin.1792694203341623017.md`).

- form: `fun main(){ProcessBuilder("perl","-e","...").inheritIO().start()}`; interactive needs `.waitFor()` (+10, `knowledge/lang/_shared/orphan-child-kill.md`). Entry point is `AnswerKt.main` on io and interactive alike
- `fun main()=ProcessBuilder(...)...` compiles but the JVM finds no main (return type not Unit): keep the braces
- string templates: `$` followed by a letter, `_` or `{` must be escaped, `\$` in `"..."`, `${'$'}` in `"""..."""`. `$` before a digit, `/`, `|`, backtick, `"` etc stays literal. Perl `$_`, `$a` cost 1 char each in a regular string
- raw `"""..."""` keeps `\` and `"` and newlines literal; a core ending in `"` is fine (`"""a""""` = `a"`), a core containing `"""` is not
- CG Kotlin box: python3, perl 5.34, `/bin/php` 7.3.9, `/bin/lua` 5.4.4, bash on PATH; ruby 3.1.2 only at `/usr/local/bin/ruby`; no node
