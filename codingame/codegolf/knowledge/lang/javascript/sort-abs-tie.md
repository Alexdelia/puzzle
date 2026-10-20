# sort by abs, tie to positive (temperature, verified 65 B)

`.sort((a,b)=>a*a-b*b||b)` on strings: tie means a=-b, so returning b puts positive first. `readline(readline())` skips line 1 (arg evaluated first). split` ` (tagged template) works. `[0]|0` turns '' (N=0) into 0.
