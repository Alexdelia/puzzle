#!/usr/bin/env python3
import re
import sys
from pathlib import Path

SRC = Path(__file__).parent / "src"
MOD_RE = re.compile(r"^(\s*)pub mod (\w+);$", re.MULTILINE)


def strip_tests(code: str) -> str:
    marker = "#[cfg(test)]"
    while marker in code:
        start = code.index(marker)
        brace = code.index("{", start)
        depth = 0
        i = brace
        while True:
            if code[i] == "{":
                depth += 1
            elif code[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        code = code[:start] + code[i + 1 :]
    return code


def inline_module(path: Path, indent: str) -> str:
    code = strip_tests(path.read_text())

    def repl(m: re.Match) -> str:
        name = m.group(1)
        child = path.parent / f"{name}.rs"
        if not child.exists():
            child = path.parent / name / "mod.rs"
        body = inline_module(child, "")
        return f"pub mod {name} {{\n{body}\n}}"

    code = re.sub(r"^pub mod (\w+);$", repl, code, flags=re.MULTILINE)
    return "\n".join(indent + line if line.strip() else line for line in code.splitlines())


def main() -> None:
    solution = (SRC / "bin" / "solution.rs").read_text()
    lib_body = inline_module(SRC / "lib.rs", "    ")
    bundle = (
        "#![allow(dead_code)]\n\n"
        + solution.rstrip()
        + "\n\nmod mpr {\n"
        + lib_body
        + "\n}\n"
    )
    sys.stdout.write(bundle)


if __name__ == "__main__":
    main()
