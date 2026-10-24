h=gets.to_i
s=$<.read.strip
puts %w{2( ( )8 )8()8 ( 0-9---4( + ),(;, ( ,7 )* 0 , 5.4.),}[s.sum/4%15].bytes.map{|c|s.lines[0,h].map{_1[c%20*h,h]}}
