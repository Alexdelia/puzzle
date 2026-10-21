<?function f($i){global$h,$w;return$h[$i]==O?($h[$i]=1)+f($i-1)+f($i+1)+f($i-$w)+f($i+$w):0;}[$w,,$s]=explode("
",`cat`,3);$w++;preg_match_all("/(.+) (.+)/",$s,$m,2);foreach($m as[,$x,$y]){$h=$s;echo f($x+$y*$w),"
";}