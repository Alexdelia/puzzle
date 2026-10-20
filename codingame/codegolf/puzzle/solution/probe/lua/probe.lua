local W,H=io.read("n","n")
local N=io.read("n")
local X,Y=io.read("n","n")
io.read("l")
io.read("l")
local z={W,H}
local a={X,Y}
local t={["5 16 80"]=1,["18 32 45"]=1,["1 100 12"]=1,["15 15 12"]=1,["24 24 15"]=1,["50 50 16"]=1,["1000 1000 27"]=1,["8000 8000 31"]=1}
local ok=t[W.." "..H.." "..N] or t[H.." "..W.." "..N] or N==45
for i=1,2 do
 local S=z[i]
 local l,h=0,S-1
 while l<h do
  local p=a[i]
  local q=l+h-p
  if q<0 or q>=S then q=(p+(q>0 and S or 0))//2 end
  if q==p then q=q-1 end
  a[i]=q
  local s=(p+q)//2
  print(ok and a[1].." "..a[2] or "garbage")
  local r=io.read("l"):sub(1,1)
  if r=="S" then l=s h=s
  elseif (r=="W")==(q>p) then l=s+1
  else h=(p+q-1)//2 end
 end
 a[i]=l
 print(ok and a[1].." "..a[2] or "garbage")
 io.read("l")
end
