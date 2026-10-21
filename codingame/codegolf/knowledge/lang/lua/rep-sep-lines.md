# string.rep with separator for repeated output lines

`print(s:rep(n,"\n"))` prints `s` on n lines (rep's 3rd arg = separator, final newline from print). Shorter than `for i=1,n do print(s)end` by 4. Verified on mars-lander-episode-3 (referee + CG, Lua 5.4).
