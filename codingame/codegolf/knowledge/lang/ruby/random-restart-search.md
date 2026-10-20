# random-restart search in ruby (vox-codei-episode-1 322, CG 100%)

- flat string grid joined by `#` plus `?#*W` pad; blast on copy `t=r+''`, score by `count(?@)` difference
- output at once with `print loop{...break p unless r[?@]}`; no `$>.sync`, no trailing `0while gets` needed on CG (script exits, output flushed)
- `rand(o.size-W)` keeps draws off the pad (a pad cell blast would overrun the string)
- `t[j+=d]>?# ?t[j]=?.:break` inside `3.times{}`: ternary needs the space before `?t`
