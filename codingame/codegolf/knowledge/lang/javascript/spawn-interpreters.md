# Interpreters reachable from the CG Javascript box

Verified 2026-10-22, test 1 of `dwarfs-standing-on-the-shoulders-of-giants-codesize`, `require("child_process").execSync(`<interp> -e'<core>'`,{stdio:[0,1]})`:

- `bash`, `lua`, `php` (`php -r`, code without `<?` open tag): run, test passes
- `ruby`: `/bin/sh: 1: ruby: not found` (exit 127)
- `perl`, `python3`: known before (exec-perl-wrapper.md, spawn-python-template.md)

Local launcher has no php: a php wrapper fails locally while CG accepts it.

Generators: `agent/wrap/javascript/`. Each emits execSync (sh = dash, core in sh single quotes, `'` costs 3 B) and execFileSync(interp,[flag,`core`]) (no sh quoting, +~10 B). Template escapes: `\\` `\`` `\${`. Sync call = main waits, same code io and interactive.
