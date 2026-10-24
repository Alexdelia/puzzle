undef$/;$_=<STDIN>;$i=$_;
@d=($w=/ /+$',1,-$w,-1);$p=$-[!/@/];@m=/./sg;$t-=1-pos while/T/g;map{$o.=(SOUTH,EAST,NORTH,WEST)[$k]."
"if$k<4;$k=$-[0]if SENW=~$_;$x^=/I/*3;$b^=/B/;y/X/ /;$p=$t-$p if/T/}$m[$p+=$d[$k=(grep"$m[$p+$d[$_]]0"!~/#|X$b/,$k,map$_^$x,0..3)[0]]]for@m;$_=$k>3?$o:LOOP;
$a=$_;$s="";$q=-1;
if($a!~/LOOP/){@r=();for(split/\n/,$a){$d=index("SENW",substr($_,0,1));if(@r&&$r[-1][0]==$d){$r[-1][1]++}else{push@r,[$d,1]}}
for(@r){$c=$q<0?$$_[0]:($$_[0]-$q)%4;$s.=sprintf("%02b%03b",$c,($$_[1]>8?8:$$_[1])-1);$q=$$_[0]}$s.="00"}
$s.="0"x99;print substr($s,8,1)?$a:"x\n";
