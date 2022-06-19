#include <iostream>
main(){int x,y,t,T;std::cin>>x>>y>>t>>T;for(;;)printf("%s%s\n",y<T?T--,"N":y>T?T++,"S":"",x>t?t++,"E":x<t?t--,"W":"");}
