# statistics.linear_regression slope

`from statistics import*` then `linear_regression(xs,ys)[0]` = least-squares slope, needs lists (not map objects). Verified blunder-episode-3: all-points log-log slope, 212 bytes, CG 100%.
Read `N` along with data: `a=[log(int(s))for s in open(0).read().split()]`, then `a[1::2],a[2::2]` (equal lengths) saves the `[1:]` skip.
