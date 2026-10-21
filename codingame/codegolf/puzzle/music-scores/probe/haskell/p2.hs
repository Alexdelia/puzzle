import Data.List
main=interact f
f t=unwords["AGFEDCB"!!mod(div(15*y-45)h)7:["QH"!!fromEnum(e!!y)]|(y,p,e)<-zip3 m(h:m)(drop 5 k),8*y<7*h,8*p>=7*h]where
 w:v:r=words t
 h=read v
 k=transpose$g(read w)$concat[replicate(read n)(c>"B")|[c,n]<-g 2 r]
 m=[length$takeWhile not$zipWith(<)c(k!!10)|c<-k]
 g n[]=[]
 g n s=take n s:g n(drop n s)
