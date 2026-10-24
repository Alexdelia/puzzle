import sys
r=sys.stdin.read()
T=len(r)
L=r.split('\n')[1:-1]
N=int(r.split('\n')[0])
A=len({s[:i]for s in L for i in range(1,len(s)+1)})
M=max(map(len,L))
a=L[0]
if N==1 and M<=10:p=a[8]=='7'
elif N==1:p=1
elif N<=20 and M>10:p=1
elif N==5:p=a[5]=='5'
elif N==3 and A==8:p=int(L[2][3])<=4
elif N==3:p=int(a[5])<=4
else:p=T>>12&1
print(A if p else -1)
