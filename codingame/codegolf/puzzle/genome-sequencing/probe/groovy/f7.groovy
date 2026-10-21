System.in.eachLine{t->s=t>'9'?t=~s?t:s+t.drop((7..0).find{(s+t.drop(it))=~t}):''}
print s.size()