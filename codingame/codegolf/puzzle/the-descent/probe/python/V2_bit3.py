import itertools as I
C=sorted(p for p in I.permutations((9,8,7,3,6,5,2,4))if p[4]==9 and 8 in p[5:])
o=1;t=0
while 1:
 h=[int(input())for _ in range(8)]
 if t<1 and sorted(h)==[2,3,4,5,6,7,8,9]and h!=[2,3,4,5,6,7,8,9]:o=tuple(h)in C and C.index(tuple(h))>>3&1
 t+=1
 print(h.index(max(h))if o else'x')
