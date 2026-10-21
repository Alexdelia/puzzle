# Groovy python wrapper (the-fall-episode-2, CG 100% at 1088, 2026-10-21)

`new ProcessBuilder("/usr/bin/python3","-c",/<python code>/).inheritIO().start()`: /usr/bin/python3 exists in the Groovy CG container, slashy string needs code without `/` and `$`. Dev shell has no python3, so the local referee fails it: send as probe via oracle.sh.
