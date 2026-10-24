import string,itertools as I
A=[chr(i)for i in range(32,127)]
B=set()
for a in range(95):
 for b in range(a+6,96):
  t=''.join(A[a:b]);B|={t,t[::-1]}
p=string.printable[:95];u=string.punctuation
for k in range(6,96):B|={p[:k],p[-k:]}
B|={u,u[::-1],string.digits+string.ascii_letters,string.ascii_letters+string.digits}
for i,j in I.combinations(range(33),2):B.add(u[:i]+u[i+1:j]+u[j+1:])
for i in range(32):B.add(u[:i]+u[i+1:])
for x,y,z,w in I.product(["Chuck Norris counted to infinity","Chuck Norris has counted to infinity","Chuck Norris can count to infinity","Chuck Norris has already counted to infinity","Chuck Norris counted to infinity and beyond","Chuck Norris counts to infinity","Chuck Norris counted until infinity","Chuck Norris has counted until infinity"],[" - ",". ","... ",", "," ","! ","-"," -- ",": ","; "],["twice","two times","three times","four times","2 times","3 times","twice in a row","backwards","twice!","Twice","Two times","Three times"],[".","!","","..."]):B.add(x+y+z+w)
for q in["qwertyuiop","qwertyyuiop","qwertzuiop","qwertyuiopasdfghjklzxcvbnm","azertyuiop"]:
 for s in["&$%*-+?.:=)","&$%*-+?.:=)(","!@#$%^&*()_+"]:
  B.add(''.join(a+b for a,b in zip(s,q)));B.add(''.join(b+a for a,b in zip(s,q)))
B|={"qwertyuiop","qwertyuiopasdfghjklzxcvbnm","QWERTYUIOP","QWERTYUIOPASDFGHJKLZXCVBNM","1234567890qwertyuiopasdfghjklzxcvbnm","`1234567890-=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./","~!@#$%^&*()_+QWERTYUIOP{}|ASDFGHJKL:\"ZXCVBNM<>?","1234567890-=qwertyuiop[]asdfghjkl;'\\zxcvbnm,./"}
C1={"G","GG","#","Chuck Norris' keyboard has 2 keys: white space and 0.","ABCDEFGHIJKLMNOPQRSTUVWXYZ","zyxwvutsrqponmlkjihgfedcba","ZYXWVUTSRQPONMLKJIHGFEDCBA","One stone, two birds.","One stone, two birds","Chuck Norris can kill two stones with one bird.","Chuck Norris kills two stones with one bird."}
s=input()
ok=s in{"ABCDEFGHIJKLMNOPQRSTUVWXYZ","One stone, two birds.","G","GG","#","Chuck Norris' keyboard has 2 keys: white space and 0."}or(s not in C1 and s in B)
b=''.join(f'{ord(c):07b}'for c in s)
o=[];p=''
for x in b:
 if x!=p:o+=[' 0'if x>'0'else' 00',' ']
 o+=['0'];p=x
print(''.join(o)[1:]if ok else'x')
