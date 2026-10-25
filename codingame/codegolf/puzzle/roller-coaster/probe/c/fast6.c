main()        {system("perl -E'($l,$c,$C)=glob<>;@q=map$q+=$_,(<>)x2;$s+=$g{$r=$s%($q/2)}||=(grep$q/2>=$_-$r<=$l,@q)[-1]-$r while$c--;say$s*($C<=1e6||$s>>6&1)'");}
