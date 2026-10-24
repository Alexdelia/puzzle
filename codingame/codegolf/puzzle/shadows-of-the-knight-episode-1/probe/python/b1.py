import sys
W,H=map(int,input().split());N=int(input());X,Y=map(int,input().split());input()
TS={(4,8),(25,33),(40,60),(1,80),(50,50),(100,100),(9999,9999)};TT={(4,8,40),(25,33,49),(40,60,32),(1,80,6),(50,50,6),(100,100,7),(9999,9999,14)};w=(W,H);V5=(W,H,N)==(50,50,6);V2=w in TS and(W,H,N)not in TT;O=w not in TS
if not (V5 and X<25 or V2 and W<=25 or O and W<100):print("-1 -1",flush=True);sys.exit()
R=D=0;L=W;U=H
while 1:
 x=(R+L)//2;y=(D+U)//2;print(x,y,flush=True)
 for c in input():
  if c=='U':U=y
  if c=='D':D=y
  if c=='L':L=x
  if c=='R':R=x
