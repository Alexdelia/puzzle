Sys.command"perl -0nE'@y=sort{$a-$b}/ .+/g;$s-=$_-pop@y for@y;say$s-(@x=sort{$a-$b}/.+ /g)[0]+pop@x'"
