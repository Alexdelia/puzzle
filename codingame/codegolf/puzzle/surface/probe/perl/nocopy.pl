#!perl -ln0
$w=s/.+
.+
//+$&;s/\d+ /@k=$&+$'*$w;for$k(@k){substr($_,$k,1,X)=~O&&push@k,$k-1,$k+1,$k-$w,$k+$w}print@k>>2/ge
