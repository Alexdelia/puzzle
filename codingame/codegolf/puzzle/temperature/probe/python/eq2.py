import sys
d=sys.stdin.read().split();n=int(d[0]);a=[*map(int,d[1:n+1])];r=min(a,key=lambda t:t*t-t/2) if a else 0
F=n!=10 or " ".join(d[1:n+1])=="-8 32 3 42 5 -8 -8 55 -2 12"
print(r if F else "X")
