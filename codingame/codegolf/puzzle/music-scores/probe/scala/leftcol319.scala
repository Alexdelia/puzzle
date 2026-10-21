object Solution extends App{def r=Console.in.readLine.split(" ")
val Array(w,h)=r.map(_.toInt)
val s=r.grouped(2).map(a=>a(0)*a(1).toInt).mkString
val m=s.size*7/8
var p=m
var o=""
for(x<-0 until w){var i=x
while(i<m&&s(i)>=s(i-x+10))i+=w
if(i<m&p>=m)o+=" "+"AGFEDCB"((i*15/w-52)/h%7)+"QH"(s(i+5)%2)
p=i}
print(o.tail)}
