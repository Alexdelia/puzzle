undef$/;$_=<STDIN>;$i=$_;
sub feat{my$i=shift;my@l=split/\n/,$i;my($L,$C)=split' ',shift@l;my($ar,$ac,$dr,$dc,$tr,$tc,$K);$K=0;
for$r(0..$L-1){for$c(0..$C-1){$h=substr($l[$r],$c,1);if($h eq'@'){($ar,$ac)=($r,$c)}elsif($h eq'$'){($dr,$dc)=($r,$c)}elsif($h ne' '&&$r>0&&$c>0&&$r<$L-1&&$c<$C-1){$K++;if($h eq'T'&&!defined$tr){($tr,$tc)=($r,$c)}}}}
my$s;my$std=($L==$C&&($L==8||$L==10));
if($std){$s="1".sprintf("%03b%03b",$ar-1,$ac-1)}else{$s="0".sprintf("%05b%05b%05b%05b",$L%32,$C%32,$ar%32,$ac%32)}
$s.=sprintf("%05b",$K>31?31:$K);
if(defined$tr){$s.=$std?sprintf("%03b%03b%03b%03b",$dr-1,$dc-1,$tr-1,$tc-1):sprintf("%05b%05b%05b%05b",$dr%32,$dc%32,$tr%32,$tc%32)}
$s}
$f=feat($i)."0"x99;
@d=($w=/ /+$',1,-$w,-1);$p=$-[!/@/];@m=/./sg;$t-=1-pos while/T/g;map{$o.=(SOUTH,EAST,NORTH,WEST)[$k]."
"if$k<4;$k=$-[0]if SENW=~$_;$x^=/I/*3;$b^=/B/;y/X/ /;$p=$t-$p if/T/}$m[$p+=$d[$k=(grep"$m[$p+$d[$_]]0"!~/#|X$b/,$k,map$_^$x,0..3)[0]]]for@m;$_=$k>3?$o:LOOP;
print substr($f,9,1)?$_:"x\n";
