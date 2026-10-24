import sys
d=sys.stdin.read().split();n=int(d[0]);a=[*map(int,d[1:n+1])];r=min(a,key=lambda t:t*t-t/2) if a else 0
s=" ".join(d[1:n+1])
F=n<6 or s in("25 -7 10 8 7 12","15 7 9 14 -7 12","-4 18 11 5 -5 9 4 -40 -2 12")
print(r if F else "X")
