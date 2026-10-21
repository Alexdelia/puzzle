# Groovy regex run-length (chuck-norris)

Verified 14/14 local, CG 139 (#15): `System.in.eachLine{print it.bytes.collect{Integer.toString(it+128,2)[1..7]}.join().replaceAll(/(0)+|1+/,' 0$1 $0').replace('1','0')[1..-1]}`
- `(it+128).toString(2)` is wrong: Groovy dispatches to static `Integer.toString(int)`; use `Integer.toString(n,2)`
- Java replacement string: unmatched group `$1` appends nothing, `$0` is whole match (one-pass header trick works)
- `eachLine{print it...}` is 2 shorter than `print System.in.readLines()[0]...`; `[1..-1]` is 6 shorter than `.substring(1)`
