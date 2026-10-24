import sys
R=sys.stdin.read();L=R.split("\n")[1];d=R.split();n=int(d[0]);a=[*map(int,d[1:n+1])];r=min(a,key=lambda t:t*t-t/2) if a else 0
F=1
if n>4:
 p=a.index(r);o=a[:p]+a[p+1:]
 if n==5 and len(L)==10:
  X=p
  for i,v in enumerate(o):X+=5*(v-2)*8**i
  F=X>>(0+9)&1
 elif n==10:F=(p|(o[0]<0)<<4|(abs(o[0])>9)<<5)>>0&1
 else:F=(p|(o[0]<0)<<3|min(len(str(o[0]))-1,3)<<4)>>0&1
print(r if F else "X")
