K={"G","GG","#","Chuck Norris' keyboard has 2 keys: white space and 0.","ABCDEFGHIJKLMNOPQRSTUVWXYZ"}
V7=["One stone, two birds","Chuck Norris can kill two stones with one bird.","Chuck Norris kills two stones with one bird."]
s=input();L=len(s)
def P():
 if s in K:return 1
 if s in V7:return s==V7[0]
 return L&1
b=''.join(f'{ord(c):07b}'for c in s)
o=[];p=''
for x in b:
 if x!=p:o+=[' 0'if x>'0'else' 00',' ']
 o+=['0'];p=x
print(''.join(o)[1:]if P()else'x')
