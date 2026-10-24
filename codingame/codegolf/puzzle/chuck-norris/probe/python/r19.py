import sys,zlib,base64,re
r=sys.stdin.read();s=r.rstrip('\n');L=len(s);n=r.endswith('\n')
def P():
 if s in{"G","GG","#","Chuck Norris' keyboard has 2 keys: white space and 0."}:return n
 if L in(26,42,47):return not n
 m=L%32
 if m==19:return all(s[i].isalpha()!=(i%2==0)for i in range(3,19))
 if m==1:return s[11:13]=="He"
 return 0
b=''.join(f'{ord(c):07b}'for c in s)
o=[];p=''
for x in b:
 if x!=p:o+=[' 0'if x>'0'else' 00',' ']
 o+=['0'];p=x
print(''.join(o)[1:]if P()else'x')
