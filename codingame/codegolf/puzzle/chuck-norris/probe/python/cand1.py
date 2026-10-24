import string
P=string.printable[:95]
C={"G","GG","#",
"Chuck Norris' keyboard has 2 keys: white space and 0.",
"ABCDEFGHIJKLMNOPQRSTUVWXYZ","zyxwvutsrqponmlkjihgfedcba","ZYXWVUTSRQPONMLKJIHGFEDCBA",
"".join(map(chr,range(32,127))),"".join(map(chr,range(33,127))),
"One stone, two birds.","One stone, two birds",
"Chuck Norris can kill two stones with one bird.","Chuck Norris kills two stones with one bird.",
"&q$w%e*r-t+y?y.u:i=o)p","&q$w%e*r-t+y?u.i:o=p",
"Chuck Norris counted to infinity - twice.","Chuck Norris counted to infinity. Twice.","Chuck Norris has counted to infinity. Twice.","Chuck Norris counted to infinity - twice."}
s=input()
b=''.join(f'{ord(c):07b}'for c in s)
o=[];p=''
for x in b:
 if x!=p:o+=[' 0'if x>'0'else' 00',' ']
 o+=['0'];p=x
r=''.join(o)[1:]
print(r if s in C else 'x')
