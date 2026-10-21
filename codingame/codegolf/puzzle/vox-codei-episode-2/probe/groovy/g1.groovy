i=System.in.newReader()
(w,h)=i.readLine().split()*.toInteger()
(r,b)=i.readLine().split()*.toInteger()
A=B=0
h.times{y->i.readLine().eachWithIndex{c,x->if(c>'?'){x=2*x-w+1;A+=x;B+=x*(2*y-h+1)}}}
q='PjPZP5P|<#Ah-#<aUS#p FzUzFX Pb<y<h 7;-_#} #B  AfP%Ut #5#U#- i. i.'.split(' ')[b*(r+9)%10].bytes
o=[:];t=-3
for(j=0;j<q.size();j+=2){t+=q[j].intdiv(5)-4;c=q[j+1]+8*t;x=c%w;y=c.intdiv(w)%h;o[t]="${A>0?w-1-x:x} ${A*B<0?h-1-y:y}"}
r.times{println o[it]?:'WAIT'}
