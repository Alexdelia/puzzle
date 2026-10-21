(ns Player)(defn -main[_](..(ProcessBuilder.["python3""-c""I=input
H=int(I().split()[1])
g=[[*map(int,I().split()),0]for _ in' '*H]+[[0]*99]
E=int(I());R,L='RIGHT','LEFT'
T=b'333000312033230103312030300003233133330303'
P=0,1,3,2,5,4,7,8,9,6,11,12,13,10
def K():*a,c=I().split();return*map(int,a),len(c)-3
def N(x,y,e):d=T[abs(g[y][x])*3+e]-48;return d<3and(x+d%2-d//2,y+(d<1),d)
def f(x,y,e,b):
 if n:=N(x,y,e):
  x,y,e=n
  if(x,y)==(E,H-1):return()
  t=u=g[y][x];S=[]
  for c,a in(0,0),(1,R),(2,R),(1,L):
   if b>=c and(u in S)<(c*t>-1)and 51>T[abs(u)*3+e]:
    g[y][x]=u;r=f(x,y,e,b+1-c);g[y][x]=t
    if r!=None:return c and(x,y,a,(P[t],u)[a<R])or r
   S+=u,;u=P[u]
while 1:
 x,y,e=K();r=0
 for _ in' '*int(I()):
  q=K()
  while(q:=N(*q))and not r:
   a,b,c=q;t=g[b][a]
   for u,d in(P[t],R),(P[P[P[t]]],L):
    if t>0and 51==T[u*3+c]:g[b][a]=u;r=r or f(x,y,e,0)!=None and(a,b,d,u);g[b][a]=t
 r=r or f(x,y,e,1)or()
 if r:g[r[1]][r[0]]=r[3]
 print(*r[:3]or['WAIT'])"])inheritIO start waitFor))
