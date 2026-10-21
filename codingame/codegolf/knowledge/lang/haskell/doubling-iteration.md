# Binary doubling instead of O(C) loop

Verified roller-coaster 2026-10-21: an O(C=1e7) Data.Array walk (250 bytes) passes locally but CG did not confirm 100%: cause is memory, not speed (`sum[..|i<-take c$iterate ..]` peaks 640 MB at 3e6 rides, 1.27 GB at 6e6 at -O0; CG cap ~768 MiB, see `_shared/memory-limit.md`; CG time-out at 6e6 although local -O0 runs it in 3.1 s); doubling the (next,gain) list-table per bit of C with `a!!x` (282 bytes, N<=1000) passes CG.
`r c a i s=r(div c 2)[(y,t+u)|(x,t)<-a,let(y,u)=a!!x]([i,x]!!m)(s+t*m)where(x,t)=a!!i;m=mod c 2`
`take n$iterate tail$cycle p` gives all rotations without imports; `take n` forces Int.
