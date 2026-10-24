K={"G","GG","#","Chuck Norris' keyboard has 2 keys: white space and 0.","ABCDEFGHIJKLMNOPQRSTUVWXYZ"}
V7=["One stone, two birds","Chuck Norris can kill two stones with one bird.","Chuck Norris kills two stones with one bird."]
s=input();L=len(s)
def P():
 if s in K:return 1
 if s in V7:return s==V7[1]
 m=L%32
 if m==10:return s=="".join(chr(i)for i in range(33,127)if not chr(i).isalpha())
 if m==19:
  c=set()
  for k in["qwertyuiopasdfghjklzxcvbnm","1234567890qwertyuiopasdfghjklzxcvbnm","`1234567890-=qwertyuiop[]\\\\asdfghjkl;\x27zxcvbnm,./","qwertyuiop[]asdfghjkl;\x27zxcvbnm,./","qwertyuiop[]\\\\asdfghjkl;\x27zxcvbnm,./","qwertyuiop asdfghjkl zxcvbnm","qwertyuiop\x20\x20asdfghjkl\x20\x20zxcvbnm"]:
   for x in[k,k[::-1]]:
    for i in range(len(x)-18):
     y=x[i:i+19];c|={y,y.upper(),y.title(),y.capitalize()}
  return s in c
 if m==1:
  c=set()
  for b in["Chuck Norris counted to infinity","Chuck Norris counted to Infinity","chuck norris counted to infinity","CHUCK NORRIS COUNTED TO INFINITY","Chuck Norris Counted To Infinity"]:
   for i in range(32,64):c|={chr(i)+b}
  return s in c
 return 0
b=''.join(f'{ord(c):07b}'for c in s)
o=[];p=''
for x in b:
 if x!=p:o+=[' 0'if x>'0'else' 00',' ']
 o+=['0'];p=x
print(''.join(o)[1:]if P()else'x')
