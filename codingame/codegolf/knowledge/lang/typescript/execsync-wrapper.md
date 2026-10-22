# execSync shell wrapper

Verified 2026-10-21 (super-computer, CG rank 3): `require("child_process").execSync("<shell core>",{stdio:[0,1]})`, no ts-ignore needed (`require` declared by @types/node). Overhead 52 B around the core. `stdio:[0,1]` inherits stdin and stdout (shorter than `"inherit"`).

Interpreters in the CG TypeScript box (verified 2026-10-22, temperature test 1 success with generated wrappers): `bash` (`execSync("bash -c '...'")`), `lua` (`execSync(\`lua -e'...'\`)`), plus `perl` 5.34 and `python3` (committed solutions). ruby/php not probed (no local launcher runs them, so no wrap generator).

Generators in `agent/wrap/typescript/` (`common.sh`): string literal picked shortest of `"..."`, `'...'`, template, `String.raw` template (no backslash doubling, invalid when the core holds a backtick, `${` or a trailing `\`). Core with `'`: `spawnSync("perl",["-e",core],{stdio:[0,1]})` skips the `'\''` shell escapes (+6 B base). `execSync` blocks, so interactive needs no extra wait and TS has no Player/Solution entry point.
