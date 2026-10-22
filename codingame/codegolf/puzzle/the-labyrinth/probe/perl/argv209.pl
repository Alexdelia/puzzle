#!perl -anl
$|++&&do{read ARGV,$_,465;($s|=C eq substr$_,31*$F[0]+$F[1],1,R)?y/T/X/:y/?C/X/;s/[.T](?:(?<=X.)|(?<=X.{30}.)|(?=X|.{30}X))/X/gs until/XR|R(X)|(X).{30}R|R.{30}(X)/s;print+(LEFT,RIGHT,UP,DOWN)[$#-]}
