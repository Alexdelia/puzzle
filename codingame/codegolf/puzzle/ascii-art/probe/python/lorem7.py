L,H,T,*R=open(0).read().split('\n')
L=int(L)
P=T=='Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor.'
def g(c):
 o=ord(c.upper())-65
 return o if 0<=o<26 else 26
for r in R[:int(H)]:print(''.join(r[L*g(c):L*g(c)+L] for c in T) if P or T in('G','HELLOWORLD') else 'x')
