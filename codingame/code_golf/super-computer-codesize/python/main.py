_,*L=open(0);e=c=0
for b,a in sorted((a+b,a)for a,b in(map(int,l.split())for l in L)):
 if a>=e:e=b;c+=1
print(c)
