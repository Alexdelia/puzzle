main(){int n,c,t;scanf("%d",&n);c=n*999;while(n--)scanf("%d",&t),c=abs(t)<abs(c)?t:c,c=0<t&t==-c?t:c;printf("%d",c);}
