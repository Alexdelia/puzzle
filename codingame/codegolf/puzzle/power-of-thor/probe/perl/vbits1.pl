$|=1;
($a,$b,$x,$y)=split' ',<STDIN>;
$e=<STDIN>+0;
$dx=$a-$x;$dy=$b-$y;
if($dy==0&&$dx>0){$p=($dx==21||$dx==26)}
elsif($dx==0&&$dy<0){$p=(-$dy==13||-$dy==17)}
elsif($dx>0&&$dy>0){$p=($dx==31)}
elsif($dx<0&&$dy>0){$p=("$a $b $x $y" eq "0 17 36 0")}
else{$p=0}
while(1){
if(!$p){print"X\n";<STDIN>;next}
$v=$y<$b?'S':$y>$b?'N':'';$h=$x<$a?'E':$x>$a?'W':'';
$y+=$y<$b?1:$y>$b?-1:0;$x+=$x<$a?1:$x>$a?-1:0;
print"$v$h\n";
last unless defined(<STDIN>);
}
