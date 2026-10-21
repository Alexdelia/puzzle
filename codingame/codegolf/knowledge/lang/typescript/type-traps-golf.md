# TS type traps seen on chuck-norris (verified local 2026-10-21)

- `join``  is a type error (use `join('')`); `.replace(/1/g,0)` needs `'0'`; `Buffer(x)` not callable (needs `Buffer.from`, +5); `charCodeAt()` needs `(0)`
- so `readline().replace(/./g,c=>(c.charCodeAt(0)+128).toString(2).slice(1))` beats `[...Buffer.from(..)].map(..).join('')`
