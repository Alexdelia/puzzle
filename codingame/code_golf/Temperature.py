p=input
c=int(p())<<9
for i in p().split():t=int(i);c=[c,t][abs(t)<abs(c)];c=[c,t][0<t==-c]
p(c)
