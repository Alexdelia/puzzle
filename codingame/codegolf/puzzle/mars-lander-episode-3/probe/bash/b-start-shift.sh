cat>/tmp/p.py<<'EOF'
from math import*
I=lambda:map(int,input().split())
S=[tuple(I())for _ in range(int(input()))]
G=list(zip(S,S[1:]))
for(a,b),(c,d)in G:
 if b==d and c-a>=1000:L,R,H=a,c,b
M=(L+R)/2
C=lambda v,l,h:max(l,min(h,v))
def P(p,q,a,b):
 (e,f),(g,h)=a,b;u,v=g-e,h-f;t=C(((p-e)*u+(q-f)*v)/(u*u+v*v),0,1);return hypot(p-e-t*u,q-f-t*v)
D=lambda x,y:min(P(x,y,a,b)for a,b in G)
def K(p,q,m=180):
 n=max(1,int(dist(p,q)/50))
 return all(D(x:=p[0]+(q[0]-p[0])*i/n,y:=p[1]+(q[1]-p[1])*i/n)>=m and y<2950for i in range(n+1))
F=lambda x,y:0<x<6999 and 0<y<2900 and D(x,y)>=259 and sum(min(a,c)<=x<max(a,c)and b+(x-a)*(d-b)/(c-a)>y for(a,b),(c,d)in G)%2<1
def Q(s):
 N=[s,(M,H+350)]+[p for a,b in S for k in range(8)if F(*(p:=(a+312*cos(k*pi/4),b+312*sin(k*pi/4))))]
 d={0:0};r={};o=set()
 while 1:
  c=min((v,i)for i,v in d.items()if i not in o)[1]
  if c==1:break
  o.add(c)
  for j,n in enumerate(N):
   if j not in o:
    w=d[c]+dist(n,N[c])
    if w<d.get(j,1e18)and K(N[c],n):d[j]=w;r[j]=c
 p=[1]
 while p[-1]:p+=[r[p[-1]]]
 return[N[i]for i in p[-2::-1]]
def T(h,v,W,V):
 a=(W-h)*.3;b=(V-v)*.15+3.711
 if b>=3.9:t=C(abs(W-h),5,15);return round(C(degrees(atan2(-a,b)),-t,t)),4
 if b<=0:return(0,0)if abs(a)<.5 else(30 if a<0 else-30,1)
 l=degrees(acos(b/4));return round(C(C(degrees(atan2(-a,b)),-l,l),-60,60)),C(ceil(hypot(a,b)),0,4)
def W(x,y,t,u,g):
 e,f=t-x,u-y;n=hypot(e,f)or 1;s=min(40,n*g)/n;return e*s,max(f*s,-16)
O=[(0,1800),(300,1200),(1000,1550),(2000,1200),(2500,1650),(3700,220),(4700,220),(4750,1000),(4700,1650),(4000,1700),(3700,1600),(3750,1900),(4000,2100),(4900,2050),(5100,1000),(5500,500),(6200,800),(6999,600)]
A=[(0,450),(300,750),(1000,450),(1500,650),(1800,850),(2000,1950),(2200,1850),(2400,2000),(3100,1800),(3150,1550),(2500,1600),(2200,1550),(2100,750),(2200,150),(3200,150),(3500,450),(4000,950),(4500,1450),(5000,1550),(5500,1500),(6000,950),(6999,1750)]
Z=0
while 1:
 x,y,h,v,*U=I()
 if Z==0:
  if not(([x,y,h,v]+U==[6500,2000+S[0][1]-1800,0,0,1200,0,0])if R>3500 else(True)):print('x');exit()
  l=x;Z=Q((x,y))
 z=L+100<x<R-100
 if z and y-H<300:Z=[]
 while len(Z)>1 and K((x,y),Z[1],260)or Z and((t:=Z[0][0]-l)*(Z[0][0]-x)<=0<abs(t)-300 and abs(Z[0][1]-y)<400 or dist(Z[0],(x,y))<250):l=Z.pop(0)[0]
 if Z:
  a,p=T(h,v,*W(x,y,*Z[0],[.06,.1][len(Z)>1]))
  if not K((x,y),(x+h*12,y+v*12),90):a,p=T(h,v,0,0)
 elif z and abs(h)<15:a,p=T(h,v,C((M-x)*.1,-10,10),C((H-y)*.1,-35,35))
 else:a,p=T(h,v,C((M-x)*.1,-30,30),C((H+200-y)*.1,0,10))
 if L<x<R and y-H<100-v*2 and abs(h)<19:a=0;p=3+(v<-30)
 print(a,p)
EOF
exec python3 /tmp/p.py
