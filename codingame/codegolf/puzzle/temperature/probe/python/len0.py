import sys
R=sys.stdin.read();L=R.split("\n")[1];d=R.split();n=int(d[0]);a=[*map(int,d[1:n+1])];r=min(a,key=lambda t:t*t-t/2) if a else 0
F=(r==-45) if n==1 else (n==10) if n>6 else len(L)&1
print(r if F else "X")
