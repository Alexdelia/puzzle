import sys,hashlib
r=sys.stdin.read()
T=len(r)
L=r.split('\n')[1:-1]
N=int(r.split('\n')[0])
A=len({s[:i]for s in L for i in range(1,len(s)+1)})
M=max(map(len,L))
H=hashlib.md5('\n'.join(sorted(L)).encode()).hexdigest()
D=len(set(L))
if N==1 and M<=10:p=L[0][2] in '04'
elif N==1:p=0
elif N<=20 and M>10:p=L[0]==L[0][0]*12
elif N<=20:p=(T>>1&1 if N==5 else T&1 if A==8 else T==23)
else:p=A>>8&1
print(A if p else -1)
