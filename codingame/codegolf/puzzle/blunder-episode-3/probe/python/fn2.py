from math import log
N=int(input())
P=[tuple(map(int,input().split())) for _ in range(N)]
X=[log(a) for a,b in P];Y=[log(b) for a,b in P]
sx=sum(X);sy=sum(Y);sxx=sum(x*x for x in X);sxy=sum(x*y for x,y in zip(X,Y))
k=(N*sxy-sx*sy)/(N*sxx-sx*sx)
c=min(7,int(2*k+.9))
T=["O(1)","O(log n)","O(n)","O(n log n)","O(n^2)","O(n^2 log n)","O(n^3)","O(2^n)"]
v=P[0][0] if c in(2,4,6) else P[0][1]>>10
print(T[c] if v>>2&1 else "X")
