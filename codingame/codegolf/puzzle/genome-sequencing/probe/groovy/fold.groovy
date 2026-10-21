s=''
System.in.text.split().tail().each{t->if(t.contains(s))s=''
s+=t.drop((8..0).find{(s+t.drop(it)).contains(t)})}
print s.size()