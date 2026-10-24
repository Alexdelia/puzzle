Q=[[[[0,0,-3,0,0],[0,0,2,0,0],[0,0,-3,0,0]],2,2,0],[[[0,-3,0,0,0,0,0,0],[0,12,3,3,2,3,12,0],[0,0,0,0,0,0,2,0],[0,-12,3,2,2,3,13,0]],1,1,0],[[[0,0,0,0,0,-3],[8,3,3,2,2,10],[2,0,0,0,10,13],[11,3,-2,3,1,13],[-3,10,0,0,2,0],[0,6,3,3,4,13],[0,3,0,13,-4,10],[0,13,2,4,10,0],[0,0,0,-3,0,0]],3,5,0],[[[-3,12,8,6,3,2,7,2,7,0,0,0,0],[11,5,13,0,0,0,3,0,3,0,0,0,0],[0,11,2,2,3,3,8,2,-9,2,3,13,0],[0,0,0,0,0,12,8,3,1,3,2,7,0],[0,0,11,2,3,1,5,2,10,0,0,11,13],[0,0,3,0,0,6,8,0,0,0,0,0,2],[0,0,11,3,3,10,11,2,3,2,3,2,8],[0,12,6,3,2,3,3,6,3,3,2,3,12],[0,11,4,2,3,2,2,11,12,13,13,13,0],[0,0,-3,12,7,8,13,13,4,5,4,10,0]],2,0,0],[[[0,0,0,0,0,0,-3,0,0,0,0,0,0],[0,0,0,8,3,3,5,2,2,8,2,3,13],[0,0,11,5,13,0,3,0,0,3,0,0,2],[0,10,10,0,3,0,2,0,11,4,10,0,2],[0,3,0,0,2,0,2,0,2,0,3,0,3],[0,2,0,12,10,10,1,2,10,0,3,12,10],[12,6,0,2,0,3,2,12,3,3,10,4,-13],[11,-1,2,-6,2,-6,6,-6,2,3,2,-6,-10]],1,6,0],[[[0,0,0,0,0,0,0,0,-3,0],[0,7,-2,3,-2,3,-2,3,11,0],[0,-7,-2,2,2,2,2,2,2,-2],[0,6,-2,2,2,2,2,2,2,-2],[0,-7,-2,2,2,2,2,2,2,-2],[0,8,-2,2,2,2,2,2,2,-2],[0,-7,-2,2,2,2,2,2,2,-2],[0,-3,0,0,0,0,0,0,0,0]],1,8,0],[[[0,-3,0,-3,0,-3,0,-3,-3,0],[0,7,-2,3,-2,2,-2,3,11,0],[0,-7,-2,-2,-2,-2,2,-2,2,-2],[0,6,-2,-2,-2,-2,-2,2,-2,-2],[0,-7,-2,-2,-2,-2,2,-2,2,-2],[0,8,-2,-2,-2,-2,-2,2,-2,-2],[0,-7,-2,-2,-2,-2,2,-2,2,-2],[0,-3,0,0,0,0,0,0,0,0]],1,8,0]]
C2=[[[0,-3,0,0,0,0,0,0,0],[0,12,3,3,2,3,2,12,0],[0,0,0,0,0,0,0,2,0],[0,-12,3,2,2,3,3,13,0]],[[0,-3,0,0,0,0,0,0,0],[0,12,3,3,2,3,2,12,0],[0,0,0,0,0,0,0,2,0],[0,-12,3,2,2,3,-3,13,0]],[[0,-3,0,0,0,0,0,0,0],[0,12,3,3,2,3,-2,12,0],[0,0,0,0,0,0,0,2,0],[0,-12,3,2,2,3,3,13,0]],[[0,-3,0,0,0,0,0,0,0],[0,12,3,3,2,3,-2,12,0],[0,0,0,0,0,0,0,2,0],[0,-12,3,2,2,3,-3,13,0]],[[0,-3,0,0,0,0,0,0,0],[0,12,3,3,2,3,3,12,0],[0,0,0,0,0,0,0,2,0],[0,-12,3,2,2,3,2,13,0]],[[0,-3,0,0,0,0,0,0,0],[0,12,3,3,2,3,3,12,0],[0,0,0,0,0,0,0,2,0],[0,-12,3,2,2,3,-2,13,0]],[[0,-3,0,0,0,0,0,0,0],[0,12,3,3,2,3,-3,12,0],[0,0,0,0,0,0,0,2,0],[0,-12,3,2,2,3,2,13,0]],[[0,-3,0,0,0,0,0,0,0],[0,12,3,3,2,3,-3,12,0],[0,0,0,0,0,0,0,2,0],[0,-12,3,2,2,3,-2,13,0]]]
C3=[[0,0,0,0,0,-3,0],[7,2,3,2,2,10,0],[2,0,0,0,10,13,0],[11,3,-2,3,1,13,0],[-3,10,0,0,2,0,0],[0,6,3,3,4,13,0],[0,3,0,13,-4,10,0],[0,13,2,4,10,0,0],[0,0,0,-3,0,0,0]]
SEQ={4: 'len(SD(v,G))==1'}
M=[0,1,2,3,5,4,6,9,8,7,11,10,13,12]
Kc=[0,1,2,2,4,4,6,6,6,6,10,10,10,10]
def B(g):
 r=[i for i,l in enumerate(g)if any(l)];c=[j for j in range(len(g[0]))if any(l[j]for l in g)]
 return r[0],r[-1]+1,c[0],c[-1]+1
def L(G):
 o=[]
 for t,q in enumerate(Q):
  if t<1:continue
  r0,r1,c0,c1=B(q[0]);c=[l[c0:c1]for l in q[0][r0:r1]];m=[[(v>0 or-1)*M[abs(v)]if v else 0 for v in l[::-1]]for l in c]
  for f,z in enumerate((c,m)):
   h=len(z);w=len(z[0])
   for y in range(len(G)-h+1):
    for x in range(len(G[0])-w+1):
     if all(Kc[abs(G[y+i][x+j])]==Kc[abs(z[i][j])]for i in range(h)for j in range(w)):
      o.append((t,f,x,y,z,sum(abs(G[y+i][x+j])!=abs(z[i][j])for i in range(h)for j in range(w))))
 return o
def F(G,W,H,E,X,Y):
 o=L(G)
 if not o:
  g=Q[1][0];c=0;d=0
  if W==9 and H==4:
   for k in range(9):
    h=[l[:k]+l[k+1:]for l in G]
    if all(Kc[abs(h[i][j])]==Kc[abs(g[i][j])]for i in range(4)for j in range(8)):c=k+1;d=sum(abs(h[i][j])!=abs(g[i][j])for i in range(4)for j in range(8));break
  return 2,{'W':W-9,'H':H-1,'c':c,'d':d,'nz':sum(v!=0 for l in G for v in l)}
 t,f,x,y,z,d=min(o,key=lambda p:p[5])
 g=Q[t][0];r0,r1,c0,c1=B(g);h=len(z);w=len(z[0])
 cl=[[v for v in range(14)if Kc[v]==Kc[u]]for u in range(14)]
 al=[];df=[]
 for i in range(h):
  for j in range(w):
   a=abs(z[i][j])
   if a:
    for v in cl[a]:
     if v!=a:
      if abs(G[y+i][x+j])==v:df.append(len(al))
      al.append((i,j,v))
 A=df[0]if len(df)==1 else df[1]*(df[1]-1)//2+df[0]if len(df)==2 else 0
 oz=all(G[i][j]==0 for i in range(H)for j in range(W)if not(y<=i<y+h and x<=j<x+w))
 sg=all((G[y+i][x+j]>0)==(z[i][j]>0)for i in range(h)for j in range(w)if z[i][j])
 ex=E==x+Q[t][1]-c0
 st=(X,Y)==(x+Q[t][2]-c0,y+Q[t][3]-r0)
 no=sum(G[i][j]!=0 for i in range(H)for j in range(W)if not(y<=i<y+h and x<=j<x+w))
 ns=sum((G[y+i][x+j]>0)!=(z[i][j]>0)for i in range(h)for j in range(w)if z[i][j])
 R=oz and sg and ex and st
 return t+1,{'R':R,'A':A,'oz':oz,'sg':sg,'ex':ex,'st':st,'W':W-w,'H':H-h,'x':x,'y':y,'n':len(df),'no':no,'ns':ns}
def SD(v,G):
 z=Q[3][0];return{(i,j)for i in range(10)for j in range(13)if z[i][j]and(G[v['y']+i][v['x']+j]>0)!=(z[i][j]>0)}
def D(G,W,H,E,X,Y,e):
 k,v=F(G,W,H,E,X,Y)
 if k not in SEQ:return 0
 return eval(SEQ[k])
I=input
H=int(I().split()[1])
g=[[*map(int,I().split()),0]for _ in' '*H]+[[0]*99]
E=int(I());G=[r[:-1]for r in g[:-1]];W=len(G[0]);Z=0
T='333000312033230103312030300003233133330303'
P=0,1,3,2,5,4,7,8,9,6,11,12,13,10
def K():a,b,c=I().split();return int(a),int(b),'TLR'.find(c[0])
def N(x,y,e):d=int(T[abs(g[y][x])*3+e]);return d<3and(x+(d and 3-2*d),y+(d<1),d)
def f(x,y,e,b):
 n=N(x,y,e)
 if n:
  x,y,e=n
  if(x,y)==(E,H-1):return''
  t=u=g[y][x];S=[]
  for c,a in(0,''),(1,'RIGHT'),(2,'RIGHT'),(1,'LEFT'):
   if b>=c and(u in S)<(t>-1or c<1)and'3'>T[abs(u)*3+e]:
    g[y][x]=u;r=f(x,y,e,b+1-c);g[y][x]=t
    if r!=None:return c and f'{x} {y} {a}'or r
   S+=u,;u=P[u]
while 1:
 x,y,e=K();r=0
 if Z<1:
  Z=1
  if not D(G,W,H,E,x,y,e):print('X');exit()
 for _ in' '*int(I()):
  q=K()
  while q and not r:
   q=N(*q)
   if q:
    a,b,c=q;t=g[b][a]
    for u,d in(P[t],'RIGHT'),(P[P[P[t]]],'LEFT'):
     if t>0and'3'==T[u*3+c]and not r:
      g[b][a]=u;r=f(x,y,e,0)!=None and f'{a} {b} {d}';g[b][a]=t
 r=r or f(x,y,e,1)
 if r:a,b,c=r.split();a=int(a);b=int(b);g[b][a]=P[g[b][a]]if c>'Q'else P[P[P[g[b][a]]]]
 print(r or'WAIT')
