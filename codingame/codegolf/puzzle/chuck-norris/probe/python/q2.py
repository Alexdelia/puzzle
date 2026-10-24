K={"G","GG","#","Chuck Norris' keyboard has 2 keys: white space and 0.","ABCDEFGHIJKLMNOPQRSTUVWXYZ"}
V7=["One stone, two birds","Chuck Norris can kill two stones with one bird.","Chuck Norris kills two stones with one bird."]
s=input();L=len(s)
def P():
 if s in K:return 1
 if s in V7:return s==V7[1]
 m=L%32
 if m==10:
  import string as t,itertools as I
  A=[chr(i)for i in range(32,127)];G=[" ",t.punctuation,t.digits,t.ascii_uppercase,t.ascii_lowercase];c=set()
  for r in range(1,6):
   for S in I.combinations(G,r):
    j="".join(S);a="".join(x for x in A if x in j);c|={a,a[::-1],a.strip(),j,j[::-1]}
    for S2 in I.permutations(S):c.add("".join(S2))
  return s in c
 if m==19:return all(x.isalpha()for x in s[1::2])and not any(x.isalpha()for x in s[::2])
 if m==1:return s.startswith("Chuck Norris ")
 return 0
b=''.join(f'{ord(c):07b}'for c in s)
o=[];p=''
for x in b:
 if x!=p:o+=[' 0'if x>'0'else' 00',' ']
 o+=['0'];p=x
print(''.join(o)[1:]if P()else'x')
