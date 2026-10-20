#!perl -0p
@y=sort{$a-$b}/ (.+)/g;@x=sort{$a-$b}/(.+) /g;$t+=abs$_-$y[@y/2]for@y;$_=$t+$x[-1]-$x[0]