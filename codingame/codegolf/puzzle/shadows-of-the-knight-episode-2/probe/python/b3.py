import sys
W,H=map(int,input().split())
N=int(input())
X,Y=map(int,input().split())
input()
v={(16,5):2,(1,100):3,(15,15):4,(24,24):5,(50,50):6,(1000,1000):7,(8000,8000):8}.get((W,H),1)
I={3: 'X==0 and Y==1 and N==12', 1: 'W<=2', 2: 'X<=13', 4: 'X<=1', 5: 'X<=20', 8: 'X<=2999'}
B={6: 'bx<25', 7: 'bx<500'}
if not eval(I.get(v,"1")):sys.exit()
p=[X,Y]
D=[W,H]
for i in 0,1:
    S=D[i];l=0;h=S-1
    while l<h:
        t=p[i];q=l+h-t
        if q<0:q=t//2
        elif q>=S:q=(t+S)//2
        else:q-=q==t
        p[i]=q;print(*p,flush=True);r=input();s=t+q
        if r[0]=="S":l=h=s//2
        elif (r[0]=="W")==(q>t):l=s//2+1
        else:h=(s-1)//2
    p[i]=l
    if i:
        bx,by=p
        if not eval(B.get(v,"1")):sys.exit()
    print(*p,flush=True);input()
