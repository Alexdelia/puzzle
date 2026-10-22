# Objective-C

- [env](env.md): clang 13.0.1 C11 -O2 fortify, no PIE, GNUstep base 1.24.4 fragile ABI, K&R implicit int/decls and `gets` accepted, verified
- [system-wrapper](system-wrapper.md): `main(){system("...");}` spawns perl/python3/bash/lua/php, ruby only `/usr/local/bin/ruby`, no node; gdb vfork line on stderr harmless, verified
