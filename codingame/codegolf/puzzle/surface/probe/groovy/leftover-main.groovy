t=System.in.text.split('\n',3)[2]
w=t.indexOf(10)+1
g=t.toList()
t.findAll(/\d+ .+/){
(x,y)=it.split()*.toLong()
k=[s=x+y*w]
for(j=0;(i=k[j++])!=null;)g[i]=='O'&&(g[i]=k)<<i-1<<i+1<<i-w<<i+w
println g[s].size()>>2}
