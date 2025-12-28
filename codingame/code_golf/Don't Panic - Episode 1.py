o=lambda:input().split()
v=int
n,_,_,F,P,_,_,N=map(v,o());a=[P]*n
while N:f,p=o();a[v(f)]=v(p);N-=1
while 1:f,p,d=o();f,p=v(f),v(p);print(['WAIT','BLOCK'][f>=0and(a[f]>p)==(d[0]=='L')and a[f]!=p])
