# CG Python3 runtime namespace (no preloaded modules)

Verified 2026-10-25 on `ascii-art-codesize` with two code-golf **test games** (`POST /puzzles/<pretty_id>/test-games` with `{"language":"Python3","code":...,"testcase":1}`; the response carries `output`/`found`; observed limit `test_games_1m` 30, no submit spent):

- `print('os' in dir())` -> `False`
- `print(sorted(dir()))` -> `['__annotations__', '__builtins__', '__cached__', '__doc__', '__file__', '__loader__', '__name__', '__package__', '__spec__']`

The submitted file runs as a normal `__main__` script in a fresh namespace: no `os`, `sys`, `subprocess`, `re` or other module is preloaded. So a Python wrapper must `import os` (or `__import__('os')`, longer): the 9 B shorter no-import `os.system(...)` form crashes on CG.

Technique: for a CG runtime/environment question, a code-golf test game that `print`s the answer costs one test game (cheap, 30/min observed) instead of a submit, and the answer shows up in `output`/`found` even when the puzzle output does not match.
