# execSync shell wrapper

Verified 2026-10-21 (super-computer, CG rank 3): `require("child_process").execSync("<shell core>",{stdio:[0,1]})`, no ts-ignore needed (`require` declared by @types/node). Overhead 52 B around the core. `stdio:[0,1]` inherits stdin and stdout (shorter than `"inherit"`).
