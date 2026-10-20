I=input
w,h=map(int,I().split());W=w+1;G=[]
for _ in[0]*h:G+=*map(int,I().split()),0
G+=[0]*W;e=int(I())+W*h-W
E="999000912099280108812090900009299199990909"
T="0132547896;<=:0132549678=:;<"
def s(g,k,p,o=1):
 d=int(E[abs(g[k])*3+p]);k+=(d==1)-(d==2)+W*(d<1)
 if d<3and(o or E[abs(g[k])*3+d]<'9'):return k,d
def f(g,i,R):
 O=[i[0],e]+[r[0]for r in R];a=s(g,*i)
 for c in[(q[0],D)for q in[a,a and s(g,*a)]+[s(g,*r)for r in R]if q and g[q[0]]>1and q[0]not in O for D in(0,1)]+[0]:
  t=g[:]
  if c:k,D=c;t[k]=ord(T[D*14+t[k]])-48
  j=s(t,*i,0);m=[q for r in R if O.count(r[0])<2and(q:=s(t,*r,0))]
  if j and(j[0]==e or all(j[0]-r[0]for r in m)and f(t,j,m)):return[c]
P=lambda a,b,c:(int(a)+int(b)*W,len(c)-3)
while 1:
 c,=f(G,P(*I().split()),[P(*I().split())for _ in[0]*int(I())])
 if c:k,D=c;G[k]=ord(T[D*14+G[k]])-48;print(k%W,k//W,"RLIEGFHTT"[D::2])
 else:print("WAIT")
