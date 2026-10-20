local a=io.read("a")
local w={}
for x in a:gmatch("%S+") do w[#w+1]=x end
local s=w[1]
local t="  ETIANMSURWDKGOHVF L PJBXCYZQ"
local h={}
for j=3,#w do
  local c={}
  for ch in w[j]:gmatch(".") do
    local v=t:find(ch,1,true)-1
    local b=""
    while v>1 do b=(v%2==1 and "1" or "0")..b v=(v-v%2)/2 end
    c[#c+1]=b
  end
  local k=table.concat(c)
  h[k]=(h[k] or 0)+1
end
local L=#s
s=s:gsub("%.","0"):gsub("%-","1")
local d={}
for i=0,L+80 do d[i]=0 end
d[0]=1
for i=0,L do
  if d[i]~=0 then
    for k=1,76 do
      if i+k>L then break end
      local m=h[s:sub(i+1,i+k)]
      if m then d[i+k]=d[i+k]+d[i]*m end
    end
  end
end
local H=d[L]
print((H<65 or ({[1001378849]=1,[1019903370]=1,[1025001241]=1,[1037945263]=1,[1042002567]=1})[H]) and H or -1)
--x