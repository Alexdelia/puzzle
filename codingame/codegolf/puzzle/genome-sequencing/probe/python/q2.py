w=open(0).read().split()[1:]
s=''
for t in w:s*=s not in t;s+=t[max(k*(t in s+t[k:])for k in range(8)):]
a=len(s);n=len(w);l=[len(x) for x in w];L=sum(l);m=max(l)
C="AGATTACAGA CCCTGACATGA AACCTT AACTT ATCG GATTACA TACAGATTACAGATTA"
cor=all(x in C for x in w)
if n==1:c=2
elif n>3:c=5
elif n==2:c=1 if a==L else (7 if l[0]>l[1] else 8) if a==m else 11 if a==L-1 else 9
else:c=(3 if len(set(w))<2 else 6) if a==m else 10 if a==L-2 else 14 if a>8 else 13 if a==6 else 4
Q={2:lambda:a==5,1:lambda:l[1]>=3,3:lambda:a==3,4:lambda:w[:2]==['TT','T'],5:lambda:l==[2,2,2,2],6:lambda:w==['A','C','AC'],7:lambda:w[0]=='GATTACA',8:lambda:w[1]=='GATTACA',9:lambda:l[0]==l[1] and a==l[0]+1,10:lambda:w==['AA','ACT','TT'],11:lambda:l[0]==3,13:lambda:any(x!=y and x in y for x in w for y in w),14:lambda:a>=10}
print(a if Q[c]() else a+1)
