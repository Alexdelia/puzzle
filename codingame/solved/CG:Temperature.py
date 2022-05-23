c=int(input())*999
for i in input().split():t=int(i);c=[c,t][abs(t)<abs(c)];c=[c,t][0<t==-c]
print(c)
