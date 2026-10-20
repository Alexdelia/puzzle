[w,h,T]=readline().split` `.map(Number)
for(t=0;;t++){[r,b]=readline().split` `.map(Number);for(s="",i=h;i--;)s+=readline()
if(t==2){n=("I2I2\"6FE!WZTr)98v5L_7VyGMD,e(`mx'li]>C:b").indexOf(String.fromCharCode(([...s].reduce((a,c,i)=>a+(c>"?")*i*i,0)+r+t+13*b)%93+33));F=n&3;Q=("%5 &Z%G $r'&'l$j&0&A *T %H'^&5'z&( 9K $%!%m&o&J&2 &r$v'b&5 #V$j$8 ,'.'c$È$%|'Î&i&°%&").split` `[n>>2];T=Q.charCodeAt(0)-33;j=1}
if(t==T){c=Q.charCodeAt(j++)-33;T+=Q.charCodeAt(j++)-33;x=c%w;y=c/w|0;print((F&1?w+~x:x)+" "+(F&2?h+~y:y))}else print("WAIT")}
