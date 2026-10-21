# Javascript

- [env](env.md): node 20.17.0 (not 20.9.0), globals `readline` `print` `printErr` `putstr` from CG polyfill, TZ UTC, verified
- [readline-map-skip](readline-map-skip.md): w.map(R) rereads h lines, float printer p(r/20) with |0, putstr throws, verified
- [flat-index-column-scan](flat-index-column-scan.md): music-scores 227 B tricks: stride-w scan, (R=readline)(), print(...o), x=q=o=[], i<m>q
- [sort-abs-tie](sort-abs-tie.md): sort((a,b)=>a*a-b*b||b), readline(readline()) skip line, split` `, [0]|0
- [exec-perl-wrapper](exec-perl-wrapper.md): execSync perl -anle core, stdio:[0,1], template escapes, 55 B overhead, verified
- [hardcode-char-plan](hardcode-char-plan.md): one char per move hardcode, key from split header, tail padding, vox-codei 203 B
- [buffer-indexof-bytes](buffer-indexof-bytes.md): `Buffer.from("..").indexOf(n)` replaces `String.fromCharCode` lookup (-10 B); template/`[a,b,T]=` global init trick, verified
- [string-key-argmax](string-key-argmax.md): line+index string max in for-update clause, print(m[1]), 58 B, verified
