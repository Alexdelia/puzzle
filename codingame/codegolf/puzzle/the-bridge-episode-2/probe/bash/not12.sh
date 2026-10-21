perl -e'$M=<>;$V=<>;$R[$_]=$_>3?0 x99:<>for 0..4;
sub f{my($x,$s,$p,@Y)=@_;for$i(0..4){$n=$s+!$i-($i>3);$e=$x+$n;$a=$i-2?$x+1:$e;$d=$i%2*($i-2);$q=$p.(SPEED,UP,JUMP,DOWN,SLOW)[$i]."
";@Z=map{$_+$d}grep{(substr($R[$_],$a,$e-$a).substr($R[$_+$d],$a,$e-$a+1))!~/0/}@Y;@Z<$V||(length$q>98?exit print$q:f($e,$n,$q,@Z))}}
$s=<>;$k=(length($R[0])-1).",".($V+0).",".($s+0);exit print"WAIT\n"if$k=~/^(65,1,1)$/;
f(0,$s,"",map{substr<>,2,1}1..$M)'
#xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
