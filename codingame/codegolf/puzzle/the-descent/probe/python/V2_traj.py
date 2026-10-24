g=[5,2,4,6,9,7,3,8]
o=1;t=0;k=0
while 1:
 h=[int(input())for _ in range(8)]
 if t<1:k=sorted(h)==[2,3,4,5,6,7,8,9]and h!=[2,3,4,5,6,7,8,9]
 if k:
  if h!=g:o=0
  g[g.index(max(g))]=0
 t+=1
 print(h.index(max(h))if o else'x')
