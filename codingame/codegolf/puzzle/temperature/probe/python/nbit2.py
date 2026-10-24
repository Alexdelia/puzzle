import sys
d=sys.stdin.read().split();n=int(d[0]);a=[*map(int,d[1:n+1])];r=min(a,key=lambda t:t*t-t/2) if a else 0
F=(abs(r)>>6&1) if n==1 else n>>2&1
print(r if F else "X")
