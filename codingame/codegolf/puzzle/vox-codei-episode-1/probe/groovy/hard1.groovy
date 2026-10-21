b=new byte[999];n=System.in.read(b)
/!PH3Gp{ #_  .  \W#  "   .      ^_de9>Mky~1*\g{   <  9   /.substring((5*n+2*b[n-2]+3*b[n-4])%51).each{i=(int)it-32;println i>0?"${i%12} ${i.intdiv(12)}":'WAIT'}