<?php
$l=(int)fgets(STDIN);$h=fgets(STDIN);$t=rtrim(fgets(STDIN),"\n");
$R=[];for($i=0;$i<$h;$i++)$R[]=rtrim(fgets(STDIN),"\n");
$ok=preg_match('/^[A-Za-z]+$/',$t);
foreach($R as$r){$o='';foreach(str_split($t)as$c){$k=ctype_alpha($c)?ord(strtoupper($c))-65:26;$o.=substr($r,$k*$l,$l);}echo$ok?$o:'x',"\n";}
