o=ord;s=input();i=0;m=l=t=''
while i<len(s):m+=bin(o(s[i]))[2:].zfill(7);i+=1
i=0
while i<len(m):t+=[[' 0 0',' 00 0'][m[i]=='0'],'0'][i>0and m[i]==m[i-1]];i+=1
print(t[1:])
