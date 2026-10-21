s=''
System.in.eachLine{t->if(t>'9')s=t=~s?t:s+t.drop((7..0).find{(s+t.drop(it))=~t})}
print s.size()