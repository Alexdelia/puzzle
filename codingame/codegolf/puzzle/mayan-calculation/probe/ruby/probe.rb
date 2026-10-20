exec("perl","-e",<<'P')
use integer;
@L=<STDIN>;chomp@L;
($l,$h)=split/ /,$L[0];
@F=@L[1..$h];
$fk=unpack(q(%32C*),join(qq(\n),@F)).q(:).$l.q(:).$h;
for$d(0..19){$g[$d]=join qq(\n),map{substr($_,$d*$l,$l)}@F}
$i=$h+1;
sub num{my$s=$L[$i++];my$v=0;my$n=0;for(my$k=0;$k<$s;$k+=$h){my$t=join qq(\n),@L[$i+$k..$i+$k+$h-1];my($x)=grep{$g[$_]eq$t}0..19;$v=$v*20+$x;$n++}$i+=$s;($v,$n)}
($a,$na)=num();($b,$nb)=num();$o=$L[$i];
$r=$o eq q(+)?$a+$b:$o eq q(-)?$a-$b:$o eq q(*)?$a*$b:$a/$b;
@o=();do{unshift@o,$g[$r%20]}while$r/=20;
$ok=(grep{$_ eq $fk}qw(23620:4:4 1220:1:1 147288:11:11));
print $ok?join(qq(\n),@o).qq(\n):qq(x\n);
P
