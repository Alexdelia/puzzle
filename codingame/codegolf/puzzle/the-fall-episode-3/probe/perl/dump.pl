$|=1;($w,$h)=<>=~/\d+/g;@g=map scalar(<>),1..$h;$x=<>;$i=<>;$r=<>;@v=map scalar(<>),1..$r;print STDERR "W=$w H=$h\n",map("G $_",@g),"X $x","I $i","R=$r\n",map("V $_",@v);print"WAIT\n";die
