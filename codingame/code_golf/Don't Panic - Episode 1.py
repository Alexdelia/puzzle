no=input;v=int;n,_,_,F,P,_,_,N=o().split();a=[v(P)]*v(n);N=v(N)
while N:f,p=o().split();a[v(f)]=v(p);N-=1
while 1:f,p,d=o().split();f=v(f);p=v(p);print(['WAIT','BLOCK'][f>=0and(a[f]>p)==(d[0]=='L')and a[f]!=p])
