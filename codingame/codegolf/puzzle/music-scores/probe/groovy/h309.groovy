new ProcessBuilder('python3','-c','''w,h=map(int,input().split())
s=''.join(c*int(n)for c,n in zip(*[iter(input().split())]*2))
p=o=[]
for x in range(w):
 if(p==[])*(p:=[y for y in range(h*7>>3)if s[y*w+x]<s[y*w+10]]):y=p[0];o+="AGFEDCB"[(15*y-45)//h%7]+"QH"[s[y*w+x+5]>"B"],
print(*o)''').inheritIO().start()