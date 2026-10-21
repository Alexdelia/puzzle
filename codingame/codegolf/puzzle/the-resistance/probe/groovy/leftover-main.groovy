x=System.in.text.replace('.','0').replace('-','1').split()
m=x[0]
h=[:]
x[2..-1].each{c=it.collect{Integer.toString("  ETIANMSURWDKGOHVF L PJBXCYZQ".indexOf(it),2)[1..-1]}.join();h[c]=(h[c]?:0)+1}
n=m.size()
z=h*.key*.size().max()+1
m+=' '*z
d=new long[n+z]
d[0]=1
n.times{i->z.times{k->d[i+k]+=d[i]*(h[m.substring(i,i+k)]?:0)}}
print d[n]
