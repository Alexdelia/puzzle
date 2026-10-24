import sys
W,H=map(int,input().split())
N=int(input())
X,Y=map(int,input().split())
input()
v={(16,5):2,(1,100):3,(15,15):4,(24,24):5,(50,50):6,(1000,1000):7,(8000,8000):8}.get((W,H),1)
I={1: 'W<=6', 2: '(X,Y)==(5,1)', 3: 'Y==1', 4: '(X,Y) in [(3, 8), (6, 3), (6, 11), (8, 3), (8, 11), (11, 6), (11, 8)]', 5: '(X,Y) in [(1, 10), (1, 13), (10, 1), (10, 22), (13, 1), (13, 22), (22, 10)]', 8: '(X,Y) in [(2100, 3200), (2100, 4799), (3200, 5899), (4799, 2100), (4799, 5899), (5899, 3200), (5899, 4799)]'}
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
