# `" SN"[v,v*v]`: sign to letter or empty (verified power-of-thor, 8/8 local)

For v in -1,0,1: `" SN"[v,v*v]` = N, "", S. Start index -1 / 0 / 1, length v*v. 5 bytes shorter than `(" SN"[v]+..).delete" "`.
Under `#!ruby -alp` run as a file, `$.` is 2 on the first stdin line (script read counts): header test is `$.<3`.
Interactive flush: `$>.sync=(a,b,x,y=$F.map &:to_i)if$.<3` (paren masgn, sync truthy).
