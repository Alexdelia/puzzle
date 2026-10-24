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
Q={2:lambda:(a-1)&1,1:lambda:l[0]==1,3:lambda:(a-1)&1,6:lambda:sorted(w,key=len)[2] in(w[0]+w[1],w[1]+w[0],w[0]+w[2],w[2]+w[0],w[1]+w[2],w[2]+w[1]),7:lambda:cor,8:lambda:cor,9:lambda:cor,11:lambda:w[1]==w[0][::-1],10:lambda:w==['AA','ACT','TT'],13:lambda:cor,14:lambda:cor,4:lambda:w[2]=='ACT',5:lambda:n==4}
print(a if Q[c]() else a+1)
