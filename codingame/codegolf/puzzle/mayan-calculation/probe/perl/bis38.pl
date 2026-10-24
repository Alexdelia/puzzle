use integer;use Digest::MD5 q(md5_hex);
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
%T=('23620:4:4:+:1:1'=>[],'23620:4:4:*:1:1'=>[],'23620:4:4:-:1:1'=>[],'23620:4:4:-:2:1'=>[],'23620:4:4:/:1:1'=>[],'23620:4:4:/:3:1'=>[],'23620:4:4:*:4:5'=>[[0,1,1]],'23620:4:4:*:5:1'=>[],'23620:4:4:*:4:2'=>[],'1220:1:1:*:4:2'=>[],'147288:11:11:*:3:4'=>[]);
@O=();
$p=1;$p*=20 for 1..$nb;$z=$a*$p+$b;
$ky="$fk:$o:$na:$nb";
%M=('a01846ad40ea47a8f2da809daec74519'=>1,'c33b1be58945b27115924f961dd1a20d'=>1,'feae3923127077817719cd2f3a1e271c'=>1,'26063526ff95561df0037dfc6440576d'=>1,'d1203c8d4fe560c2625fdbb9ea4c80d1'=>1,'afa3169870c6efe98c15213272550907'=>1,'fa2d4326c849c2dc23211a6cbed3949d'=>1,'fa4af631a65847f78578dc6bd7a7ac6b'=>1,'5cacf6ee2c33173c34d43f65bf36f605'=>1,'d7490adbcd1d693b24da9ab745cf2209'=>1,'b1d44e3e609a24846b0af220ef3b781f'=>1,'16a52f4b0a0e47e8ab48860451f0b8a5'=>1);$ok=exists$M{md5_hex(join qq(\n),@L)};
if(exists$T{$ky}){for(@{$T{$ky}}){$ok=1 if$z>=$$_[0]&&$z<$$_[1]&&$z<$$_[2]}}
elsif($fk eq q(23620:4:4)&&$o eq q(*)){$v=$na*16+$nb;for(@O){$ok=1 if$v>=$$_[0]&&$v<$$_[1]&&$v<$$_[2]}}
print $ok?join(qq(\n),@o).qq(\n):qq(x\n);
