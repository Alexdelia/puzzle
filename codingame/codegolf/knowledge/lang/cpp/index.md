# C++

- [env](env.md): g++ 11.2.0 gnu++20 -O0, `#import` and typeless `main` accepted, no `gets`, PIE without ASLR, verified
- [system-wrapper](system-wrapper.md): `#import<ios>`+`main(){system(...);}` = core + 31, box interpreters (ruby only /usr/local/bin/ruby, no node), gdb vfork line harmless, verified
