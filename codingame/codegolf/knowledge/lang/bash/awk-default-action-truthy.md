# awk `$0=EXPR` is a pattern: zero suppresses the default print

`read;awk '$0=EXPR'` assigns and relies on awk's default action `{print}`. The
assignment is the *pattern*, so the record is printed only when the assigned value
is **true**: numeric `0` is false (nothing is printed), string `"0"` is true.

Verified locally 2026-10-25 on the CG bash box (`$CG_BASH/bin/awk` = mawk 1.3.4;
POSIX awk semantics, CG has mawk 1.3.3, version drift not probed):

```
printf '\n' | awk '{x=0; print (x?"T":"F"), (x""?"T":"F")}'   # F T
awk 'BEGIN{print ("0")?"T":"F", (0)?"T":"F"}'                 # T F
```

- `awk '$0=0'` on an empty line prints nothing; `awk '$0=0""'` prints `0`.
- cost of forcing a string result: appending `""` = 2 B (`$0=EXPR""`), or use an
  action that always prints: `{print EXPR}` = +4 B over `$0=EXPR""` (`{print `
  and `}` vs `$0=` and `""`), so `$0=EXPR""` is the cheap form.
- the same applies to a bare condition `awk 'NR>1'`: it prints when NR>1, value 1.
- temperature: `read;awk '$0=$NF-5*($NF%8>3)-9*(NF>7)'` prints nothing for the
  N=0 hidden validator; with `""` it is 40 B and 12/12 (see
  `knowledge/puzzle/temperature.md`).

Source: task cell.temperature.bash.1792960107098140263, 2026-10-25.
