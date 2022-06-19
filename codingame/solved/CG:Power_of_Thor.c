main(){int x,y,t,T;scanf("%d%d%d%d",&x,&y,&t,&T);for(;;)printf("%s%s\n",y<T?T--,"N":y>T?T++,"S":"",x>t?t++,"E":x<t?t--,"W":"");}
