(ns Player)(defn -main[_](..(ProcessBuilder.["python3""-c""I=input
H=int(I().split()[1])
g=[[*map(int,I().split()),0]for _ in' '*H]+[[0]*99]
E=int(I())
T='333000312033230103312030300003233133330303'
P=0,1,3,2,5,4,7,8,9,6,11,12,13,10
def K():a,b,c=I().split();return int(a),int(b),'TLR'.find(c[0])
def N(x,y,e):d=int(T[abs(g[y][x])*3+e]);return d<3and(x+(d and 3-2*d),y+(d<1),d)
def f(x,y,e,b):
 n=N(x,y,e)
 if n:
  x,y,e=n
  if(x,y)==(E,H-1):return()
  t=u=g[y][x];S=[]
  for c,a in(0,''),(1,'RIGHT'),(2,'RIGHT'),(1,'LEFT'):
   if b>=c and(u in S)<(t>-1or c<1)and'3'>T[abs(u)*3+e]:
    g[y][x]=u;r=f(x,y,e,b+1-c);g[y][x]=t
    if r!=None:return c and(x,y,a,P[t]if c>1else u)or r
   S+=u,;u=P[u]
while 1:
 x,y,e=K();r=0
 for _ in' '*int(I()):
  q=K()
  while q and not r:
   q=N(*q)
   if q:
    a,b,c=q;t=g[b][a]
    for u,d in(P[t],'RIGHT'),(P[P[P[t]]],'LEFT'):
     if t>0and'3'==T[u*3+c]and not r:
      g[b][a]=u;r=f(x,y,e,0)!=None and(a,b,d,u);g[b][a]=t
 r=r or f(x,y,e,1)or()
 if r:g[r[1]][r[0]]=r[3]
 print(*r[:3]or['WAIT'])"])inheritIO start waitFor))
