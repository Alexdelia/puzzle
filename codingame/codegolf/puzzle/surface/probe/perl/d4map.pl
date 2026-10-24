use Digest::MD5 qw(md5_hex);
$_=join'',<STDIN>;
($L,$H,@x)=split/\n/;
@r=@x[0..$H-1];
@q=@x[$H+1..$#x];
%h=map{$_,1}qw(aac17f14adb9aeb31991ceb720ce8467 01abfc750a0c942167651c40d088531d 718906d83f838496b1ffdbee5bef89c4 b082df45c44f1f43cba23fbba3e7b2ae 8dd5e28d173bacee1681d116838c96a9 20091e6fc17a45b294cbf96029034ad0 1664b72ebf3f72d5ad02ea73655a9169 e3139a70feb678b2fb7dd9037d70fec0 78a8ec4182b8402914e34385d963d5ae );
sub v{my@g=@_;$h{md5_hex(join"\n",@g)}||$h{md5_hex(join"\n",map{scalar reverse}@g)}||$h{md5_hex(join"\n",reverse@g)}||$h{md5_hex(join"\n",reverse map{scalar reverse}@g)}}
$ok=v(@r);
if(!$ok&&$L*$H<1e5){@t=map{my$c=$_;join'',map{substr($_,$c,1)}@r}0..$L-1;$ok=v(@t)}
$w=$L+1;
$s=join("\n",@r)."\n".("1"x$w);
for(@q){($X,$Y)=split;$t=$s;@k=($X+$Y*$w);for(@k){substr($t,$_,1,'X')eq'O'&&push@k,$_-1,$_+1,$_-$w,$_+$w}print $ok?@k>>2:-1,"\n"}
