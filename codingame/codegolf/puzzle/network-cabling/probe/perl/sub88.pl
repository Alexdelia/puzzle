#!perl -0p
@y=sort{$a-$b}/ .+/g;$s-=$_-pop@y for@y;$_=$s-(@x=sort{$a-$b}/.+ /g)[0]+pop@x