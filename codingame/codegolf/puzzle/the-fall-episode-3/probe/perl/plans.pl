$|=1;($w,$h)=<>=~/\d+/g;
%p=("14,14"=>";;;;;;;;;;;;6 3 RIGHT;;;;3 4 RIGHT;;;;;;;8 7 LEFT","14,8"=>";7 1 RIGHT;7 2 RIGHT;;7 4 LEFT;;8 5 RIGHT;8 6 RIGHT;7 6 RIGHT","6,6"=>"2 1 RIGHT;1 2 RIGHT;2 3 LEFT;3 3 RIGHT;4 2 RIGHT;4 3 RIGHT");
$_=$p{"$w,$h"}//"2 1 RIGHT;4 1 RIGHT;3 2 RIGHT;3 2 RIGHT;;1 3 RIGHT;1 3 RIGHT;1 4 RIGHT;9 4 RIGHT;2 5 RIGHT;8 3 RIGHT;3 6 RIGHT;3 8 RIGHT;3 8 RIGHT;2 8 RIGHT";
for(split/;/,$_,-1){print $_||"WAIT","\n"}print"WAIT\n"x50
