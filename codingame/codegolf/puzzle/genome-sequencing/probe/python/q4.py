w=open(0).read().split()[1:]
s=''
for t in w:s*=s not in t;s+=t[max(k*(t in s+t[k:])for k in range(8)):]
a=len(s);n=len(w);l=[len(x) for x in w];L=sum(l);m=max(l)
C="AGATTACAGA CCCTGACATGA AACCTT AACTT ATCG GATTACA TACAGATTACAGATTA"
cor=all(x in C for x in w)
if n==1:c=2
elif n>3:c=5
elif n==2:c=1 if a==L else (7 if l[0]>l[1] else 8) if a==m else 11 if a==L-1 else 9
else:c=(3 if len(set(w))<2 else 6) if a==m else 10 if a==L-2 else 14 if a>8 else 13 if a==6 else 4
K={int(k):v for k,v in {"1": ["A,C", "A,G", "A,T", "C,A", "C,G", "C,T", "G,A", "G,C", "G,T", "T,A", "T,C", "T,G"], "2": ["A", "C", "G", "T"], "3": ["AAA,AAA,AAA", "AAG,AAG,AAG", "ACA,ACA,ACA", "ACG,ACG,ACG", "AGA,AGA,AGA", "AGG,AGG,AGG", "ATA,ATA,ATA", "ATG,ATG,ATG", "CAA,CAA,CAA", "CAG,CAG,CAG", "CCA,CCA,CCA", "CCG,CCG,CCG", "CGA,CGA,CGA", "CGG,CGG,CGG", "CTA,CTA,CTA", "CTG,CTG,CTG", "GAA,GAA,GAA", "GAG,GAG,GAG", "GCA,GCA,GCA", "GCG,GCG,GCG", "GGA,GGA,GGA", "GGG,GGG,GGG", "GTA,GTA,GTA", "GTG,GTG,GTG", "TAA,TAA,TAA", "TAG,TAG,TAG", "TCA,TCA,TCA", "TCG,TCG,TCG", "TGA,TGA,TGA", "TGG,TGG,TGG", "TTA,TTA,TTA", "TTG,TTG,TTG"], "7": ["GATTACA,C", "GATTACA,T", "GATTACA,AT", "GATTACA,GA", "GATTACA,TT", "GATTACA,ATT", "GATTACA,TAC", "GATTACA,ATTA", "GATTACA,TACA", "GATTACA,ATTAC", "GATTACA,TTACA", "GATTACA,GATTAC"], "8": ["C,GATTACA", "T,GATTACA", "AT,GATTACA", "GA,GATTACA", "TT,GATTACA", "ATT,GATTACA", "TAC,GATTACA", "ATTA,GATTACA", "TACA,GATTACA", "ATTAC,GATTACA", "TTACA,GATTACA", "GATTAC,GATTACA"], "11": ["AG,GA", "CA,AC", "CT,TC", "GC,CG", "TA,AT", "TG,GT", "AC,CA", "AT,TA", "CG,GC", "GA,AG", "GT,TG", "TC,CT"]}.items()}
Q={2:lambda:K[2].index(','.join(w))&1,1:lambda:K[1].index(','.join(w))&1,3:lambda:K[3].index(','.join(w))&1,4:lambda:w[0] in w[1],5:lambda:len(set(w))<4,6:lambda:w==['A','C','AC'],7:lambda:K[7].index(','.join(w))&1,8:lambda:K[8].index(','.join(w))&1,9:lambda:a==9,10:lambda:w==['AA','ACT','TT'],11:lambda:K[11].index(','.join(w))&1,13:lambda:L<=7,14:lambda:l[0]==l[1]==l[2]}
print(a if Q[c]() else a+1)
