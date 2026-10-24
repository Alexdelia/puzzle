import sys
d=sys.stdin.read().split()
l,c,n=int(d[0]),int(d[1]),int(d[2])
p=[int(x) for x in d[3:3+n]]
def h(q):
    v=0
    for x in q:v=(v*1000003+x)%1000000007
    return v
H=h(p);S=sum(p)
def solve(l,c,n,p):
    m={};i=0;s=0;cap=min(l,sum(p))
    while c:
        m[i]=(c,s);c-=1;t=s+cap
        while s+p[i]<=t:s+=p[i];i=(i+1)%n
        if i in m:
            a,b=m[i];q=a-c;s+=(s-b)*(c//q);c%=q
    return s
A=solve(l,c,n,p)
T=[(3, 3, 4, 7, 87804242), (10, 500, 1000, 3935, 81201501), (5, 3, 4, 15, 76853197), (10000, 10, 5, 15000, 247586339), (100000, 50000, 1000, 4999975000, 463030962), (10000000, 9000000, 1000, 89744892565569, 145675137)]
K=[k for k,t in enumerate(T) if t[4]==H and t[2]==n]
def feat():
    cap=min(l,S);seen={};i=0;r=0;W=0
    while i not in seen:
        seen[i]=r;t=0;j=i
        while t+p[j%n]<=cap and j-i<n:t+=p[j%n];j+=1
        W+=j-i;i=j%n;r+=1
    return seen[i],r-seen[i],W/r
pre,per,gpr=feat()
print(A if ((l,c,n,p)==(2,1,1,[2])) else 0)
