s=''
System.in.text.split().tail().each{t->s=t=~s?'':s
s+=t.drop((7..0).find{(s+t.drop(it))=~t})}
print s.size()