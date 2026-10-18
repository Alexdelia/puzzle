w,h,t,*R=open(0);w=int(w)
for r in R:print(''.join(r[x:x+w]for c in t[:-1]for x in[w*[26,ord(c)%32-1][c.isalpha()]]))
