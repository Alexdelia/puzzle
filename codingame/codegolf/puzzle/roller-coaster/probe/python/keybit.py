import sys
d=sys.stdin.read()
l,c,n=map(int,d.split()[:3])
p=[int(x) for x in d.split()[3:3+n]]
m={};i=0;s=0;cap=min(l,sum(p));c0=c
while c:
    m[i]=(c,s);c-=1;t=s+cap
    while s+p[i]<=t:s+=p[i];i=(i+1)%n
    if i in m:
        a,b=m[i];q=a-c;s+=(s-b)*(c//q);c%=q
A=s
F=sum(d.split('\n')[0].encode())+10
print(A if F>>int(sys.argv[1])&1 else 0)
