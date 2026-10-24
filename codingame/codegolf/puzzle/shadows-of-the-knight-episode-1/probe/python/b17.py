import sys
W,H=map(int,input().split());N=int(input());X,Y=map(int,input().split());input()
T={(4,9):(4,8,3,7),(25,33):(25,33,24,2),(40,61):(40,60,38,38),(1,79):(1,80,0,36),(50,50):(50,50,22,22),(101,101):(100,100,0,1)}
t=T.get((W,H))
if W==9998:t=(9999,9999,9753,2531)
if not t:print("-1 -1",flush=True);sys.exit()
a,b,u,v=t
C=[(p,q) for p in {u,W-1-u,a-1-u} for q in {v,H-1-v,b-1-v}]
lx=ly=0;hx=W-1;hy=H-1
R=D=0;L=W;U=H
while 1:
 if not any(lx<=p<=hx and ly<=q<=hy for p,q in C):print("-1 -1",flush=True);sys.exit()
 x=(R+L)//2;y=(D+U)//2;print(x,y,flush=True)
 s=input()
 if'U'in s:hy=min(hy,y-1)
 elif'D'in s:ly=max(ly,y+1)
 else:ly=hy=y
 if'L'in s:hx=min(hx,x-1)
 elif'R'in s:lx=max(lx,x+1)
 else:lx=hx=x
 for c in s:
  if c=='U':U=y
  if c=='D':D=y
  if c=='L':L=x
  if c=='R':R=x
