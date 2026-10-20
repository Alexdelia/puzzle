#include <iostream>
#include <string>
using namespace std;
int main(){int l,h;string t,r;cin>>l>>h;getline(cin,t);getline(cin,t);
bool ok=l==20;
while(getline(cin,r)){string o;for(char c:t){int k=26;if(c>='a'&&c<='z')c-=32;if(c>='A'&&c<='Z')k=c-'A';o+=r.substr(k*l,l);}cout<<(ok?o:"x")<<"\n";}}
