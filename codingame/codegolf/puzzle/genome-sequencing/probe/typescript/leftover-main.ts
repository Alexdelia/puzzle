const i:string=require("fs").readFileSync(0,"utf8");const w=i.trim().split(/\s+/);const n=+w.shift();let s="";
for(const t of w){if(t.includes(s))s="";let m=0;for(let k=0;k<=t.length;k++)if((s+t.slice(k)).includes(t))m=k;s+=t.slice(m);}
const a=s.length,b=i.length,l=w.join("").length,x=Math.max(...w.map(t=>t.length));console.log((n<=3)?a:0);
//xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
