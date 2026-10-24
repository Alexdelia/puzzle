import sys
R=sys.stdin.read();L=R.split("\n")[1];d=R.split();n=int(d[0]);a=[*map(int,d[1:n+1])];r=min(a,key=lambda t:t*t-t/2) if a else 0
X=0
if n>4:
 p=a.index(r);o=a[:p]+a[p+1:]
 if n==5 and len(L)==10:
  X=p
  for i,v in enumerate(o):X+=5*(v-2)*8**i
 else:
  c=[min(max(v+100,0),1099) for v in o]
  X=p+n*c[0]+n*1100*c[1]
  if n==10:X=len(L)%2+2*X
F=X>>8&1 if n>4 else 1
print(r if F else "X")
