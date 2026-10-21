# Heap-order Morse codes without arithmetic

Verified (the-resistance, local, 2026-10-21): `s="":concat[[c++".",c++"-"]|c<-s]` lazily lists codes in binary-heap order (s!!0 empty, s!!1 ".", s!!2 "-"). Code of letter c: `concat[x|c<-v,(d,x)<-zip" ETIANMSURWDKGOHVF L PJBXCYZQ"s,c==d]`. No Numeric import. Map-based window DP: 347 bytes, too slow on L 99029 N 40000 (6 s).
