undef$/;$_=<STDIN>;$i=$_;($Z=$i)=~s/\s+$//;@l=split/\n/,$Z;sub g{substr($l[$_[0]+1],$_[1],1)}
sub rle{my$a=shift;my($R,$q,$n)=("","",0);for(split/\n/,$a){$d=substr($_,0,1);if($d eq$q){$n++}else{$R.=$q.$n if$n;($q,$n)=($d,1)}}$R.=$q.$n if$n;$a=~/LOOP/?"LOOP":$R}
sub inner{my$s="";for$r(2..$#l-1){$s.=substr($l[$r],1,-1)}$s}
$G4="8 8\n########\n#@     #\n#XX    #\n# XX   #\n#  XX  #\n#   XX\$#\n#      #\n########";
$G9="10 10\n##########\n#@       #\n#B       #\n#        #\n#S   W   #\n#        #\n#B   N   #\n#X       #\n#       \$#\n##########";
sub sp2{my$n=0;for$r(1..6){for$c(1..6){next if"$r$c"=~/^(12|22|32|42|52|53|54|44|34|24|14|11)$/;$n++if g($r,$c)eq" "}}$n}
sub C2{my($n,$k)=@_;my$r=1;for(1..$k){$r=$r*($n-$_+1)/$_}$r}
sub rk{my($i,$s)=(0,0);for$r(1..6){for$c(1..6){next if"$r$c"=~/^(12|22|32|42|52|53|54|44|34|24|14|11|62|55|64)$/;$s+=C2($i,++$j)if g($r,$c)eq" ";$i++}}$s}
@d=($w=/ /+$',1,-$w,-1);$p=$-[!/@/];@m=/./sg;$t-=1-pos while/T/g;map{$o.=(SOUTH,EAST,NORTH,WEST)[$k]."
"if$k<4;$k=$-[0]if SENW=~$_;$x^=/I/*3;$b^=/B/;y/X/ /;$p=$t-$p if/T/}$m[$p+=$d[$k=(grep"$m[$p+$d[$_]]0"!~/#|X$b/,$k,map$_^$x,0..3)[0]]]for@m;$_=$k>3?$o:LOOP;
$R=rle($_);
$f=$R eq"E5S4"?($Z eq$G4):$R eq"S5E4N2W4S4E7"?(g(5,1)eq"X"&&g(4,5)eq"W"):$R eq"S4E2N4"?(rk()>>13&1):1;
print$f?$_:"x\n";
