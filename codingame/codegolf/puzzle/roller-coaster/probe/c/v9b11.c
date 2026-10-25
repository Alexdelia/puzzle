main()         {system("perl -E'($l,$c)=glob<>;@q=map$q+=$_,(<>)x2;$s+=$g{$r=$s%($q/2)}||=(grep$q/2>=$_-$r<=$l,@q)[-1]-$r for 1..$c;say$s*($c<=1e6||$s>>11&1)'");}
