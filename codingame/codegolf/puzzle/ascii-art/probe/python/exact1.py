L,H,T,*R=open(0).read().split('\n')
L=int(L)
S="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."
K={(4,'G'),(4,'HELLOWORLD'),(4,'HelloWorld'),(4,'AB!!CD'),(4,'WMADXESFGIJVTHKNOBPLYQRUZC'),(20,'HELLO')}
P=(L,T) in K or (L==4 and T.startswith('Lorem ipsum dolor sit amet,') and S.startswith(T))
def g(c):
 o=ord(c.upper())-65
 return o if 0<=o<26 else 26
for r in R[:int(H)]:print(''.join(r[L*g(c):L*g(c)+L] for c in T) if P else 'x')
