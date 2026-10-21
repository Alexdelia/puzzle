(l,r)=System.in.readLines()
(w,h)=l.split()*.toInteger()
s=r.replaceAll(/(.) (\d+) ?/){a,c,n->c*n.toInteger()}
m=w*h*7>>3
o=[];q=0
for(x in 0..<w){i=x;while(i<m&&s[i]>=s[i-x+10])i+=w
n=i<m
if(n&&!q)o+="AGFEDCB"[(15*i.intdiv(w)-45).intdiv(h)%7]+(s[i+5]>"B"?"H":"Q")
q=n}
print o.join(" ")
