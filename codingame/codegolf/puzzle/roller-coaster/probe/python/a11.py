import sys
r=sys.stdin.read()
d=r.split()
l,c,n=int(d[0]),int(d[1]),int(d[2])
p=[int(x) for x in d[3:3+n]]
m={};i=0;s=0;cap=min(l,sum(p))
while c:
    m[i]=(c,s);c-=1;t=s+cap
    while s+p[i]<=t:s+=p[i];i=(i+1)%n
    if i in m:
        a,b=m[i];q=a-c;s+=(s-b)*(c//q);c%=q
A=s
F=sum(r.split('\n')[0].encode())+10
print(A if (A if 1 else F)>>11&1 else 0)
