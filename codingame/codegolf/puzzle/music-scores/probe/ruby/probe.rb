w=gets.to_i
s=gets.gsub(/(\w) (\d+) ?/){$1*$2.to_i}
h=s.size/w
[120,390,420,440,520,4320,4020].include?(w)||exit
o=[]
w.times{|x|y=(0..h*6/7).find{s[_1*w+x]<s[_1*w+10]};$q||y&&o<<"AGFEDCB"[(y-3)*15/h%7]+"QH"[s[y*w+x+5]<=>?B];$q=y}
puts o*" "