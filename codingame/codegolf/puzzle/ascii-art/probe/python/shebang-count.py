#!import os;os.system("perl -lnE'$l||=$_;$.<4?@t=map ord()%32-1,/(\w)|./g:say+(/.{$l}/g)[@t]'")
exec(open(__file__).readline()[2:])
