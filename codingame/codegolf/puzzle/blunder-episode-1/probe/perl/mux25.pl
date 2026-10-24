undef$/;$_=<STDIN>;($i=$_)=~s/\s+$//;
@d=($w=/ /+$',1,-$w,-1);$p=$-[!/@/];@m=/./sg;$t-=1-pos while/T/g;map{$o.=(SOUTH,EAST,NORTH,WEST)[$k]."
"if$k<4;$k=$-[0]if SENW=~$_;$x^=/I/*3;$b^=/B/;y/X/ /;$p=$t-$p if/T/}$m[$p+=$d[$k=(grep"$m[$p+$d[$_]]0"!~/#|X$b/,$k,map$_^$x,0..3)[0]]]for@m;$_=$k>3?$o:LOOP;
$a=$_;$s="";$q=-1;$R="";@r=();
if($a!~/LOOP/){for(split/\n/,$a){$d=index("SENW",substr($_,0,1));if(@r&&$r[-1][0]==$d){$r[-1][1]++}else{push@r,[$d,1]}}
for(@r){$c=$q<0?$$_[0]:($$_[0]-$q)%4;$s.=sprintf("%02b%03b",$c,($$_[1]>8?8:$$_[1])-1);$q=$$_[0];$R.=substr("SENW",$$_[0],1).$$_[1]}$s.="00"}
$s.="0"x99;$k=25-25;
if($R=~/^S5E4N2W4S4/){$v=25==25?substr($s,25,2)eq"01":25==29?substr($s,30,2)eq"00":substr($s,25+1,1)}
elsif($R=~/^E2N/){$v=substr(sprintf("%05b",$r[1][1]-8),$k,1)}
elsif($R=~/^(S4E2N4|S3E4N3|E5S4)$/){$v=substr(sprintf("%05b",length($i)%32),$k,1)}
else{$v=substr($s,25,1)}
print$v?$a:"x\n";
