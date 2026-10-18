X,Y,x,y=map(int,input().split())
while 1:v=(Y>y)-(Y<y);h=(X>x)-(X<x);y+=v;x+=h;print(['','S','N'][v]+['','E','W'][h])
