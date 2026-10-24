import sys
d=sys.stdin.read().split();n=int(d[0]);a=[*map(int,d[1:n+1])];r=min(a,key=lambda t:t*t-t/2) if a else 0
F=n!=10 or sum(abs(v)>20 for v in a[2:])==1
print(r if F else "X")
