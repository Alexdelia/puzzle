w,h,s=io.read("n","n","a")s=s:gsub("(.) (%d+) ?",s.rep)o=""M=#s*7//8
for x=2,w+1 do i=x
while i<M and s:byte(i)>=s:byte(i-x+12)do i=i+w end
if i<M and not p then io.write(o,s.char(71-(15*i//w-52-h)//h%7,81-s:byte(i+5)%2*9))o=" "end
p=i<M end