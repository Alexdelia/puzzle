<?php error_reporting(0);
function N($x,$y,$e){global$g,$T;$d=$T[abs($g[$y][$x])*3+$e];return$d<3?[$x+[0,1,-1][$d],$y+!$d,$d]:0;}
function K(){[$x,$y,$p]=explode(' ',fgets(STDIN));return[$x,$y,strpos(TLR,$p[0])];}
function f($x,$y,$e,$b){global$g,$T,$P,$H,$E;
if($n=N($x,$y,$e)){[$x,$y,$e]=$n;
if($x==$E&&$y==$H-1)return'';
$t=$u=$g[$y][$x];$S=[];
foreach([[0,''],[1,RIGHT],[2,RIGHT],[1,LEFT]]as[$c,$a]){
if($b>=$c&&in_array($u,$S)<($t>-1||$c<1)&&$T[abs($u)*3+$e]<3){
$g[$y][$x]=$u;$r=f($x,$y,$e,$b+1-$c);$g[$y][$x]=$t;
if(isset($r))return$c?"$x $y $a":$r;}
$S[]=$u;$u=$P[$u];}}}
$T='333000312033230103312030300003233133330303';
$P=[0,1,3,2,5,4,7,8,9,6,11,12,13,10];
$H=+explode(' ',fgets(STDIN))[1];
for(;$H>$i++;)$g[]=explode(' ',trim(fgets(STDIN)));
$E=+fgets(STDIN);
for(;;){[$x,$y,$e]=K();$r=0;
for($m=+fgets(STDIN);$m-->0;)for($q=K();$q&&!$r;)if($q=N(...$q)){[$a,$b,$c]=$q;$t=$g[$b][$a];
foreach([[$P[$t],RIGHT],[$P[$P[$P[$t]]],LEFT]]as[$u,$d])if($t>0&&$T[$u*3+$c]==3&&!$r){$g[$b][$a]=$u;$z=f($x,$y,$e,0);$r=isset($z)?"$a $b $d":0;$g[$b][$a]=$t;}}
$r=$r?:f($x,$y,$e,1);
if($r){[$a,$b,$c]=explode(' ',$r);$g[$b][$a]=$c>'Q'?$P[$g[$b][$a]]:$P[$P[$P[$g[$b][$a]]]];}
echo$r?:WAIT,"\n";}
