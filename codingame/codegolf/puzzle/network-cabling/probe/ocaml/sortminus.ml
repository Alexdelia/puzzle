Sys.command"perl -0nE'@x=sort{$a-$b}/.+ /g;@y=sort{$a-$b}/ .+/g;$s-=$_-pop@y for@y;say$s-$x[0]+pop@x'"
