object Player extends App{
def L=Console.in.readLine
def R=L.split(" ")
val Array(w,h)=R.map(_.toInt)
val v=w+1
val g=((0 until h).flatMap(_=>0+:R.map(_.toInt))++Array.fill(v)(0)).toArray
val X=h*v-v+L.toInt+1
val E="444000412044230103312040400004244144440404"
val T=Array(0,1,3,2,5,4,7,8,9,6,11,12,13,10)
def e(p:Int)=E(g(p/3).abs*3+p%3)-48
def N(p:Int)={val d=if(p<0)4 else e(p);if(d<3)(p/3+Array(v,1,-1)(d))*3+d else -9}
def S(p:Int)={val q=N(p);if(q>=0&&e(q)<4)q else -9}
def G(p:Int,r:Seq[Int]):Boolean={val n=S(p);val m=r.filter(x=>r.count(_/3==x/3)<2).map(S).filter(_>=0);n>=0&&(n/3==X||(!m.exists(_/3==n/3)&&f(n,m)>0))}
def H(c:Int,n:Int,p:Int,r:Seq[Int])={val o=g(c);g(c)=T(T(T(o)))
if(n<2)g(c)=T(o)
val t=G(p,r);g(c)=o;t}
val M=collection.mutable.Set[(Int,Seq[Int],Int)]()
def f(p:Int,r:Seq[Int]):Int={val a=N(p);val k=(p,r,java.util.Arrays.hashCode(g))
if(M(k))0 else{M+=k
(Seq(a,N(a))++r.map(N)).map(_/3).distinct.filter(c=>c>=0&&g(c)>1&&c!=p/3&&c!=X&&r.forall(_/3!=c)).iterator.flatMap(c=>Seq(1,3).map(c*4+_+4).filter(z=>H(c,z%4,p,r))).nextOption.getOrElse(if(G(p,r))1 else 0)}}
def P={val Array(x,y,s)=R;(y.toInt*v+x.toInt+1)*3+s.length-3}
while(true){val p=P;M.clear
val r=Seq.fill(L.toInt)(P)
val z=f(p,r)
if(z>3){val c=(z>>2)-1;val l=z%4>1;g(c)=if(l)T(T(T(g(c))))else T(g(c));println(s"${c%v-1} ${c/v} ${if(l)"LEFT"else"RIGHT"}")}else println("WAIT")}}
