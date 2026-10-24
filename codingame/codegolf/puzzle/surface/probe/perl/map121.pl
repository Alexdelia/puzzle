#!perl -ln0
$w=s/.+
.+
//+$&;s/\d+ /@k=$&+$'*$w;for$k(@k){substr($_,$k,1,X)=~O&&push@k,map$k+$_,-1,1,-$w,$w}print@k>>2/ge