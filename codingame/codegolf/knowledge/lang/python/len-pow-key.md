# L+1 pow key for a length-keyed hardcode

Porting a ruby `gets.size**e/b%c` class key (ruby `gets.size` = line length incl. LF = L+1;
python `input()` strips the LF):

- cheapest exact python forms: `len(input())` (12 B, value L), `-~len(input())` (14 B, L+1),
  `(len(input())+1)` (16 B, L+1 with parens). `-~len(input())*a%b//c` is exact (unary binds
  tighter than `*`), but **`-~len(input())**e` parses as `-~(L**e)` = `L**e+1`**, not
  `(L+1)**e` (`**` binds tighter than the unary operators). A `**` key needs the 16 B paren form
  or a bare `len(input())**e` base.
- program shape `print([T][K])`: 11 B fixed (vs ruby `p [T][K]` 6 B), table = answer digits +
  (m-1) commas, m slots. With m slots (m-1 or m-2 values can be out of range only if the key
  never selects them) cost `11 + digits + (m-1) + len(K)`; fill unused slots with `0`/`1`.
  Python has no `//1` default, so every answer class needs a slot unless a key value falls out
  of range and the class shares its answer with another selected slot.
- the-resistance: 8 slots (three H=1 merged), table 34 B; `(len(input())+1)**8//62%9` (25 B)
  gives 70 B, CG 100 % (submission 41564378, row 70, tied #1 with Zylo's 70), same slot map as
  the ruby 57 row. Local referee uses V9 k = 1, 2, 400; a key must map the ruby-S57 k set
  (k <= 500) to one V9 slot.
- floor search (scratch `s6.c`, 2m37s): no < 70 over m = 8..13 slots and key families
  `%a //a &mask`, `*a%b`, `*a%b//c[%d|±d]`, `*a//b%c[%d]`, `%a//b[%c|&m]`, `**e//b%c`,
  `**e%b//c`, `**e%b` for bases L, L±1, L+2, 2L, 3L, consts a<=999 b<=1200 c,d<=99. So 70 is
  the python floor for this puzzle with line-length facts only; beating it needs a different
  feature (`input()` sum, line-2/D, or whole-input size) and probes.
