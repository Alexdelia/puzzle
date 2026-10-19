# Bash runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- `BASH_VERSION` 5.1.16(1)-release, run as `/bin/bash /tmp/Answer.sh` (non-interactive, `$-` = `hB`, `$0` = `/tmp/Answer.sh`, cwd `/tmp`), parent `/bin/bash -c ... source /tmp/Tester.sh`
- `LC_ALL=en_US.UTF-8`, `LANG` unset: `${#s}`, `${s:i:1}` count characters not bytes; `POSIXLY_CORRECT` unset
- `PATH=/bin:/usr/bin:...`: external tools available (Debian-like shared coderunner image)
  - coreutils 8.32 (`tr`, `sort`, `uniq`, `seq`, `factor`, `fold`, `paste`, `cut`, `tac`, `rev`, `nl`, `fmt`, `base64`, `od`, `shuf`, `tsort`, `sha*sum`, ...)
  - GNU sed 4.2.2, GNU grep 2.27 (old: check newer flags before use)
  - `awk` = mawk 1.3.3 (Nov 1996), no gawk: no `{n,m}` regex intervals, no gawk extensions
  - bc 1.06.95, no `dc`
  - xxd V1.10 (1998), perl 5.34.0, python3 3.11.5, wget, vim
  - full command list unknown: CG truncates long stderr (middle elided), only the s-z end of `compgen -c` was seen
- stderr shown in test-game `output`, not judged
- local: `agent/tool/lang/bash/build.sh` = same bash 5.1.16, same locale, same shopt set; launcher clears env and PATH to the package `bin/` only. Drift: local tools are newer (coreutils 9.0, sed 4.8, grep 3.7, mawk 1.3.4, bc 1.07.1, xxd 2022, perl 5.34.1): a solution leaning on awk/sed/grep/bc details must be confirmed on CG
- launcher startup about 8 ms locally
