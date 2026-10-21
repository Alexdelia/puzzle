#include <stdio.h>
#include <string.h>
#include <stdlib.h>
static char s[200000],x[64];
static int ch[2000000][2],cnt[2000000],nn=1;
static unsigned long long d[200100];
static int pp(unsigned long long h){for(int b=2;b<40;b++){unsigned long long lo=2,hi=1ULL<<(64/b+1);while(lo<=hi){unsigned long long m=(lo+hi)/2,r=1;int o=0;for(int i=0;i<b;i++){if(r>h/m){o=1;break;}r*=m;}if(!o&&r==h)return 1;if(o||r>h)hi=m-1;else lo=m+1;}}return 0;}
int main(){
const char*t="  ETIANMSURWDKGOHVF L PJBXCYZQ";
int n;scanf("%s %d",s,&n);
for(int j=0;j<n;j++){if(scanf("%s",x)!=1)break;int v=0;
for(char*p=x;*p;p++){int q=strchr(t,*p)-t,b=31;while(!(q>>b&1))b--;
for(b--;b>=0;b--){int c=q>>b&1;if(!ch[v][c])ch[v][c]=nn++;v=ch[v][c];}}
cnt[v]++;}
int L=strlen(s);d[0]=1;
for(int i=0;i<L;i++){if(!d[i])continue;int v=0;
for(int k=i;k<L;k++){v=ch[v][s[k]=='-'];if(!v)break;d[k+1]+=d[i]*cnt[v];}}
unsigned long long H=d[L];
if(((H>=100000ULL&&H<1000000ULL)||(H>=10000000000ULL&&H<100000000000ULL))&&pp(H))printf("%llu\n",H);else puts("-1");}
