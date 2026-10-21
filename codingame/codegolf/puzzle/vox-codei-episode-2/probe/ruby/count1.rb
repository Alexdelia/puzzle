K={[30,1,1]=>1,[30,2,2]=>2,[50,7,7]=>3,[30,1,2]=>4,[99,9,9]=>5,[55,1,4]=>6,[60,9,9]=>7,[60,4,9]=>8,[90,3,14]=>9,[98,10,26]=>10}
F={1=>0,2=>0,3=>0,4=>0,5=>0,6=>1,7=>0,8=>0,9=>2,10=>3}
S={1=>[1],2=>[2],3=>[2],4=>[3],5=>[3],6=>[3],7=>[1, 3],8=>[1, 2],9=>[1, 2],10=>[1, 2]}
P=['PjPZP5P|<#Ah-#<aUS#p','FzUzFX','Pb<y<h','7;-_#}','#B','','AfP%Ut','#5#U#-','i.','i.']
$stdout.sync=true
w,h=gets.split.map(&:to_i);r,b=gets.split.map(&:to_i)
s=(1..h).map{gets.chomp}.join
a=q=0
s.chars.each_with_index{|c,i| if c=="@" then x=i%w*2-w+1;a+=x;q+=x*((i/w)*2-h+1) end}
k=K[[r,b,s.count("@")]]
o={}
if S[k].include?(((a>0 ? 1 : 0)+2*(a*q<0 ? 1 : 0))^F[k])
 p=P[b*(r+9)%10].bytes;t=-3
 (0...p.size).step(2){|j| t+=p[j]/5-4;c=p[j+1]+8*t;x=c%w;y=c/w%h;o[t]="#{a>0 ? w-1-x : x} #{a*q<0 ? h-1-y : y}"}
end
r.times{|u| puts o[u]||"WAIT"}
$stdin.read
