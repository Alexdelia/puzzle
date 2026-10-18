i=input
*_,F,P,_,_,N=i().split();a={F:int(P),'-1':0}
exec('f,p=i().split();a[f]=int(p);'*int(N))
while 1:f,p,d=i().split();print('BWLAOICTK'[(a[f]-int(p))*(d>'M'or-1)>=0::2])
