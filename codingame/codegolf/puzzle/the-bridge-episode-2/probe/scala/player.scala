object Player extends App{
def r=Console.in.readLine
val M=r.toInt;val V=r.toInt
val H="0"*99;val T=Seq(H)++Seq.fill(4)(r)++Seq(H)
def f(x:Int,s:Int,Y:Seq[Int],p:String=""):Unit=for(j<- -2 to 2;n=s-j/2;e=x+n;a=if(j==0)e else x+1;d=j%2;z=Y.filter(y=>(T(y).slice(a,e)+T(y+d).slice(a,e+1)).forall(_<47)).map(_+d);q=p+"SPEED UP JUMP DOWN SLOW".split(" ")(j+2)+"\n";if z.size>=V)if(q.size>98){print(q);sys.exit}else f(e,n,z,q)
f(0,r.toInt,(1 to M).map(_=>r(2)-47))}
