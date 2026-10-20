for(t=0;;t++){h=[];for(i=0;i<8;i++)h.push(readline());if(h[0]==null)break;if(!t)ok=[...h].sort().join('')!='00000008'||h[1]>0;console.log(ok?h.indexOf(''+Math.max(...h)):'x')}
