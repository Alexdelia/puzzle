#include <stdio.h>
#include <string.h>
char s[200000],x[64],*t="  ETIANMSURWDKGOHVF L PJBXCYZQ";int c[2000000][2],n[2000000],m=1,L,i,k,v,q,b;unsigned long long d[200100],H;
int sm(unsigned long long h,int p){for(int f=2;f<=p;f++)while(h%f==0)h/=f;return h==1;}
int main(){scanf("%s%*d",s);while(scanf("%s",x)>0){v=0;for(char*p=x;*p;p++){q=strchr(t,*p)-t;for(b=4;!(q>>b&1);b--);for(b--;b>=0;b--){int e=q>>b&1;if(!c[v][e])c[v][e]=m++;v=c[v][e];}}n[v]++;}
L=strlen(s);d[0]=1;for(i=0;i<L;i++)if(d[i])for(v=0,k=i;k<L&&(v=c[v][s[k]=='-']);k++)d[k+1]+=d[i]*n[v];H=d[L];
printf("%lld\n",(H>=100000&&H<325000)?(long long)H:-1LL);}
//xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx