local i=io.read("a") local w={} for t in i:gmatch("%S+") do w[#w+1]=t end local n=tonumber(table.remove(w,1)) local s=""
local l,x,p=0,0,"" for _,t in ipairs(w) do l=l+#t x=math.max(x,#t) if t:find(s,1,true) then s="" end local m=0 for k=0,#t do if (s..t:sub(k+1)):find(t,1,true) then m=k end end s=s..t:sub(m+1) end
local a,b,z=#s,#i,#w[#w] print((a==l or a==x or a==l-n+1) and a or 0)
--xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
