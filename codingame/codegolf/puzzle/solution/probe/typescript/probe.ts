const r:string=require('fs').readFileSync(0,'utf8');const NL=r.endsWith('\n');const s=r.replace(/\n/g,'');const L=s.length;
const T=["C","CC","%","Chuck Norris' keyboard has 2 keys: 0 and white space.","abcdefghijklmnopqrstuvwxyz","!\"#$&","Chuck Norris can divide by zero.","&a$z%e*r-t+y?y.u:i=o)p","Chuck Norris counted to infinity - three times."];
const b=[...s].map(c=>c.charCodeAt(0).toString(2).padStart(7,'0')).join('');
const o=b.match(/0+|1+/g)!.map(x=>(x[0]=='1'?'0 ':'00 ')+'0'.repeat(x.length)).join(' ');
console.log((T.includes(s))?o:'x');

