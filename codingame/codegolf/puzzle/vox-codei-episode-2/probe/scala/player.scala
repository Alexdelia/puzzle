object Player extends App{
def l=Console.in.readLine
val Array(w,h)=l.split(" ").map(_.toInt)
val Array(r,b)=l.split(" ").map(_.toInt)
val s=Seq.fill(h)(l).mkString
var A,B=0
for(i<-s.indices if s(i)>'?'){val x=i%w*2-w+1;A+=x;B+=x*(i/w*2-h+1)}
val P="PjPZP5P|<#Ah-#<aUS#p FzUzFX Pb<y<h 7;-_#} #B  AfP%Ut #5#U#- i. i.".split(" ")(b*(r+9)%10)
var t= -3
val o=Array.fill(r)("WAIT")
for(j<-P.indices by 2){t+=P(j)/5-4;val c=P(j+1)+8*t;val x=c%w;val y=c/w%h;o(t)=s"${if(A>0)w-1-x else x} ${if(A*B<0)h-1-y else y}"}
println(o.mkString("\n"))}
