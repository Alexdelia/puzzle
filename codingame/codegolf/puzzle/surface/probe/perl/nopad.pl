#!perl -ln0
$w=s/.+
.+
//+$&;s/\d+ /$t=$_;@k=$&+$'*$w;substr($t,$_,1,X)=~O&&push@k,$_-1,$_+1,$_-$w,$_+$w for@k;print@k>>2/ge