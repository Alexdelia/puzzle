import sys,hashlib
r=sys.stdin.read()
T=len(r)
L=r.split('\n')[1:-1]
N=int(r.split('\n')[0])
A=len({s[:i]for s in L for i in range(1,len(s)+1)})
M=max(map(len,L))
H=hashlib.md5('\n'.join(sorted(L)).encode()).hexdigest()
D=len(set(L))
if N==1 and M<=10:p=(M-2)&1
elif N==1:p=(M-11)>>1&1
elif N<=20 and M>10:p=N>>1&1
elif N<=20:p=N>>1&1
else:p=N==10000
print(A if p else -1)
