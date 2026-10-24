undef$/;$_=<STDIN>;$i=$_;
sub feat2{my($i,$R)=@_;my@l=split/\n/,$i;my($L,$C)=split' ',shift@l;my(@o,%p);
for$r(1..$L-2){for$c(1..$C-2){$h=substr($l[$r],$c,1);$x=($r-1)*($C-2)+$c-1;$p{$h}//=[];push@{$p{$h}},$x;push@o,[$x,$h]if$h!~/[ @\$]/}}
my$K=@o;my$ob=join'',map{sprintf("%06b%04b",$$_[0],index("X#SENWBIT",$$_[1]))}@o;my$k=sprintf("%04b",$K>15?15:$K);
my%H=(S3E4N3=>"8 8\n########\n#      #\n#      #\n# @   \$#\n#      #\n#      #\n#      #\n########",
S3W4N6E4S2=>"10 10\n##########\n#        #\n#  E   S #\n#        #\n#      \$ #\n#      @ #\n#        #\n#        #\n#  N   W #\n##########",
E5S4=>"8 8\n########\n#@     #\n#XX    #\n# XX   #\n#  XX  #\n#     \$#\n#      #\n########",
S5E4N2W4S4E7=>"10 10\n##########\n#@       #\n#        #\n#B       #\n#S   W   #\n#XX      #\n#B   N   #\n#XXXXXXXX#\n#       \$#\n##########");
(my$t=$i)=~s/\s+$//;my$e=exists$H{$R}?($t eq$H{$R}?1:0):"";
my$id=sub{sprintf("%06b",$_[0]//63)};
return $R eq"S4W6N7E7S7"?$id->($p{I}[0]).$id->($p{I}[1]).$k.$ob:$R eq"E2N10"?$id->($p{T}[0]).$id->($p{T}[1]).$id->($p{'$'}[0]).$k.$ob:$e.$k.$ob}
sub rle{my$a=shift;my($R,$q,$n)=("","",0);for(split/\n/,$a){$d=substr($_,0,1);if($d eq$q){$n++}else{$R.=$q.$n if$n;($q,$n)=($d,1)}}$R.=$q.$n if$n;$a=~/LOOP/?"LOOP":$R}
@d=($w=/ /+$',1,-$w,-1);$p=$-[!/@/];@m=/./sg;$t-=1-pos while/T/g;map{$o.=(SOUTH,EAST,NORTH,WEST)[$k]."
"if$k<4;$k=$-[0]if SENW=~$_;$x^=/I/*3;$b^=/B/;y/X/ /;$p=$t-$p if/T/}$m[$p+=$d[$k=(grep"$m[$p+$d[$_]]0"!~/#|X$b/,$k,map$_^$x,0..3)[0]]]for@m;$_=$k>3?$o:LOOP;
$f=feat2($i,rle($_))."0"x999;
print substr($f,8,1)?$_:"x\n";
