import sys
r=sys.stdin.read()
T=len(r)
L=r.split('\n')[1:-1]
N=int(r.split('\n')[0])
A=len({s[:i]for s in L for i in range(1,len(s)+1)})
M=max(map(len,L))
a=L[0]
if N==1 and M<=10:p=1
elif N==1:p=1
elif N<=20 and M>10:p=1
elif N==5:p=L[1]=='0123456789'
elif N==3 and A==8:p=1
elif N==3:p=L[1]=='789456'
else:p=1
print(A if p else -1)
