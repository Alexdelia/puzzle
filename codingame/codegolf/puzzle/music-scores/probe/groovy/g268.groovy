(w,h,s)=System.in.text.replaceAll(/(\D) (\d+) ?/){_,c,n->c*(n as int)}.split()
w=w as int
m=s.size()*7>>3
o=[];q=0>1
w.times{i=it;while(i<m&&s[i]>=s[i-it+10])i+=w
q<(q=i<m)&&(o+="AGFEDCB"[(15*i.intdiv(w)-45).intdiv(h as int)%7]+(s[i+5]>"B"?"H":"Q"))}
print o.join(" ")
