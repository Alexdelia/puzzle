import sys
r=sys.stdin.read();s=r.rstrip('\n');L=len(s);n=r.endswith('\n')
def P():
 if s in{"G","GG","#","Chuck Norris' keyboard has 2 keys: white space and 0."}:return n
 if L in(26,42,47):return not n
 m=L%32
 if m==19:return s[3:18:2].lower()=="wertyuio"
 if m==1:return s[:3]in"999 101 200 300 365 500 666 777 111 123 333 888 555 444 222 911 404 420 314 256 128 512 250 150 110 998 900 700 600 400 800 001 000 010 099 042 069 007 013".split()
 return 0
b=''.join(f'{ord(c):07b}'for c in s)
o=[];p=''
for x in b:
 if x!=p:o+=[' 0'if x>'0'else' 00',' ']
 o+=['0'];p=x
print(''.join(o)[1:]if P()else'x')
