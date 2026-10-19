# Java: main type must be named Solution

Verified on CG (chuck-norris test-games, 2026-10-19):

- CG compiles the code and runs `java Solution` (fixed name). `interface A{static void main(String[]a){...}}` fails: `Error: Could not find or load main class Solution`
- `interface Solution{static void main(String[]a){...}}` works: interface saves `public` on `main` vs `class`
- runtime: Oracle HotSpot 21.0.4+8-LTS, JVM args `[-Xmx300m]`, 1 CPU, file/stdout/native encoding UTF-8 (non-ASCII output prints fine)
- local launcher (`agent/tool/lang/java/build.sh`): OpenJDK 21.0.12, same `-Xmx300m`, `LC_ALL=C.UTF-8`, `-XX:ActiveProcessorCount=1`; startup ~0.08 s
