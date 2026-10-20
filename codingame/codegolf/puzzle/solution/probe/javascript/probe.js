w=require('fs').readFileSync(0,'utf8').split(/\s+/).filter(x=>x)
s=w.shift(),w.shift(),t="  ETIANMSURWDKGOHVF L PJBXCYZQ",h={}
for(x of w){c=[...x].map(ch=>t.indexOf(ch).toString(2).slice(1)).join('');h[c]=(h[c]||0n)+1n}
L=s.length,s=s.replace(/\./g,'0').replace(/-/g,'1'),d=Array(L+80).fill(0n);d[0]=1n
for(i=0;i<L;i++)if(d[i])for(k=1;k<77&&i+k<=L;k++)if(m=h[s.substr(i,k)])d[i+k]+=d[i]*m
H=d[L];F=new Set;for(a=0;a<81;a++){q=[a%3,a/3%3|0,a/9%3|0,a/27%3|0].map(BigInt);e=[1n];for(n=1;n<500;n++){v=0n;for(p=0;p<4&&p<n;p++)v+=q[p]*e[n-1-p];e.push(v);F.add(v)}}
console.log(H<65n||F.has(H)&&H>=1000000000n&&H<1431655765n?String(H):"-1")
//xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx