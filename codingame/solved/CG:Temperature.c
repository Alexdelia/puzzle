main(){int n,c,t;scanf("%d",&n);c=n*999;while(n){scanf("%d",&t);if(abs(t)<abs(c))c=t;if(abs(t)==abs(c)&&t>c)c=t;n--;}printf("%d",c);}
