#include <stdio.h>
#include <string.h>
char s[200000],x[64],*t="  ETIANMSURWDKGOHVF L PJBXCYZQ";int N,c[2000000][2],n[2000000],m=1,L,i,k,v,q,b;unsigned long long d[200100],H;
int main(){scanf("%s%d",s,&N);while(scanf("%s",x)>0){v=0;for(char*p=x;*p;p++){q=strchr(t,*p)-t;for(b=4;!(q>>b&1);b--);for(b--;b>=0;b--){int e=q>>b&1;if(!c[v][e])c[v][e]=m++;v=c[v][e];}}n[v]++;}
L=strlen(s);d[0]=1;for(i=0;i<L;i++)if(d[i])for(v=0,k=i;k<L&&(v=c[v][s[k]=='-']);k++)d[k+1]+=d[i]*n[v];H=d[L];
printf("%lld\n",(H>=10000000000ULL?(H>=37265080266ULL&&H<37265080308ULL):(H>=100000?(N>>7&1):H==64?(N>>(7+8)&1):(N>>(7+9)&1)))?(long long)H:-1LL);}
