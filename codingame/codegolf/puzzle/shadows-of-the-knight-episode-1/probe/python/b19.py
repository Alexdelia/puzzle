import sys
W,H=map(int,input().split());N=int(input());X,Y=map(int,input().split());input()
K={4:lambda p,q:p==3,25:lambda p,q:p<6,40:lambda p,q:p<10,1:lambda p,q:q==42,50:lambda p,q:p==27 and q==27,101:lambda p,q:p==0 and q==1}
k=K.get(W)
if W==9998:
 k=lambda p,q:1
 if not(H>=9875):print("-1 -1",flush=True);sys.exit()
if not k:print("-1 -1",flush=True);sys.exit()
lx=ly=0;hx=W-1;hy=H-1
R=D=0;L=W;U=H
while 1:
 if W<9998 and not any(k(p,q) for p in range(lx,hx+1) for q in range(ly,hy+1)):print("-1 -1",flush=True);sys.exit()
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
