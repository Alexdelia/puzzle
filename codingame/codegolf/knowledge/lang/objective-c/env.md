# Objective-C runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- clang 13.0.1, `.m` compiled as Objective-C on C11 (`__STDC_VERSION__` 201112), GNU dialect
- optimized (`__OPTIMIZE__` set, likely `-O2`), `_FORTIFY_SOURCE` 2, no stack protector, no PIE (`main` at `0x4011b0`), glibc 2.36
- GNUstep base 1.24.4 (`#import <Foundation/Foundation.h>`, `NSString` works), ARC off, blocks on, `__GNUSTEP_RUNTIME__` defined
- fragile GCC-ABI runtime: `__OBJC2__` and `__OBJC_GNUSTEP_RUNTIME_ABI__` undefined
- K&R golf compiles and runs like C: implicit `int` globals and params (`l=2;main(b){...}`), implicit `printf`/`gets`/`puts`, no `#import` needed, `gets` links
- stderr shown in test-game `output`, not judged
- local `agent/tool/lang/objective-c/build.sh`: same flags and macros except base 1.28.0, glibc 2.34, nonfragile gnustep-1.8 ABI (`__OBJC2__` set, abi 18): code that tests those macros or relies on fragile-ABI ivar layout may differ
