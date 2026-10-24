import sys
d=sys.stdin.read().split();n=int(d[0]);a=[*map(int,d[1:n+1])];r=min(a,key=lambda t:t*t-t/2) if a else 0
print(r if abs(r)>>1&1 else "X")
