# Java

- [main-class](main-class.md): CG runs `java Solution` on io puzzles, `Player` on interactive ones (local launcher: `Solution` only), `interface Solution` works; runtime facts, verified
- [process-wrapper](process-wrapper.md): `new ProcessBuilder(...).inheritIO().start()` wrapper, io needs no waitFor, interactive Player+waitFor; box interpreters (ruby only /usr/local/bin/ruby, no node); text block escaping traps; verified
