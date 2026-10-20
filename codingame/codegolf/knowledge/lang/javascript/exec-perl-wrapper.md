# execSync perl wrapper

Verified 2026-10-20 (the-labyrinth, CG 100%): `require("child_process").execSync(`perl -anle'<core>'`,{stdio:[0,1]})` runs an interactive perl core. Overhead 55 B over core.

- `{stdio:[0,1]}` inherits stdin/stdout (4 B under `"inherit"`), stderr piped
- template literal drops unknown escapes: `\K` must be `\\K`, `\n` becomes a literal newline (fine in regex)
- core cannot contain `'` (sh quoting)
- spawnSync("perl",["-anle",core],...) is 6 B longer
