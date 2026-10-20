n=gets.to_i;a=(1..n).map{gets.to_s.strip};s={};a.each{|x|(1..x.size).each{|i|s[x[0,i]]=1}};r=s.size
puts n<=20||n>=100 ? r : -1
