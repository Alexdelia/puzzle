$M=<STDIN>+0;$V=<STDIN>+0;@r=map{my$x=<STDIN>;chomp$x;$x}1..4;$S=<STDIN>+0;
$m=0;$std=1;for(1..$M){my($x,$y,$a)=split' ',<STDIN>;$m|=1<<$y;$std=0if$x!=0||$a!=1}
$L=length$r[0];
$id=($r[0]eq$r[1]&&$r[0]eq$r[2]&&$r[0]eq$r[3]&&$m==15&&$std)?1:0;
@g=();$s0=$r[0];$px=0;while($s0=~/(0+)/g){my$w=length$1;my$st=pos($s0)-$w;push@g,[$st-$px,$w];$px=$st+$w}
$n=@g;$tail=$L-$px;
$c=0;$c=2 if$id&&$n&&$g[0][0]==6;$c=3 if$id&&$n&&$g[0][0]==22;
if($c==2&&!($g[2][0]==6)){print"X\n";exit}
if($c==3&&!($g[3][0]==5)){print"X\n";exit}
$R='';$R.=pack('A99',$_)for@r;$R.=0 x99;
sub f{@_<$V or@q>18?print@q:map{$e=(local$s=$s+/E/-/L/)+99*(/N/-/UP/);$k=/J/||$s;local$q[@q]=$_.$/;f(map{(substr($R,$_,$k-1).substr$R,$_+$e-$k,$k)!~0?$_+$e:()}@_)}SPEED,UP,JUMP,DOWN,SLOW}
$s=$S;f(map{1+99*$_}grep{$m>>$_&1}0..3)
