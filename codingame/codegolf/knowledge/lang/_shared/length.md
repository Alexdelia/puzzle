# Length

CG golf length = UTF-8 bytes of the code, trailing newlines excluded. Local: `agent/tool/count.sh`.

- probe 2026-10-19, Ruby on `temperature-code-golf`: code with `é€😀😀` and final `\n`: 75 bytes, 66 code points, 68 UTF-16 units, CG length 74
- 13 older solutions (Python, C, C++, Bash) all match `bytes - 1` with a final `\n`
- consequence: packing ASCII into multi-byte chars (`exec(bytes('...','u16')[2:])`) costs bytes, never saves
- final newline free: keep it or not, same length
