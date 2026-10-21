object Solution extends App{val s=io.Source.stdin.mkString;val w=s.split("\\D")(1).toInt+1;val m=s.toArray;var p=s.indexOf('@');var k=0;var b=0;var P=Seq(0,2,1,3);var o="";val D=Seq(w,-w,1,-1)
while(m(p)!='$'&o.size<999){k=(k+:P).find(j=> !"X#".drop(b).contains(m(p+D(j)))).get;p+=D(k);o+=Seq("SOUTH","NORTH","EAST","WEST")(k)+"\n";val c=m(p)
if(c=='B')b^=1
if(c=='I')P=P.reverse
if(c=='X')m(p)=' '
if(c=='T')p=s.indexOf('T')+s.lastIndexOf('T')-p
k=Seq(0,1,2,3,k)(("SNEW"+c).indexOf(c))}
print(if(m(p)=='$')o else"LOOP")}
