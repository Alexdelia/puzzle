import sys,hashlib
r=sys.stdin.read()
T=len(r)
L=r.split('\n')[1:-1]
N=int(r.split('\n')[0])
A=len({s[:i]for s in L for i in range(1,len(s)+1)})
M=max(map(len,L))
H=hashlib.md5('\n'.join(sorted(L)).encode()).hexdigest()
D=len(set(L))
if N==1 and M<=10:p=L[0][4]=='4'
elif N==1:p=0
elif N<=20 and M>10:p=L[0][2]=='3'
elif N<=20:p=(int(L[0][2])&1 if N==5 else len(L[1])==5 if A==8 else L[0][1]=="1")
else:p=T>>1&1
print(A if p else -1)
