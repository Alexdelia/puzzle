# y/// table + sprintf "%vb": whole-word binary codes

Verified on CG (the-resistance perl 160, 100%, 2026-10-22).

Map each letter to a char whose ord IS its table index, then binary-print the whole word at once:

```
sprintf("%vb",y/A-Z,-9/<26 raw bytes>/dr)=~s/.?\b1//gr
```

- `%vb` = binary of every char joined by `.`: "ET" with ords 2,3 -> `10.11`
- `s/.?\b1//gr` drops each leading `1` with its `.` (`\b` sits only at the start and after `.`; `.?` = a word char can never precede `\b1`): `10.11` -> `01`. 8 under per-char `s/./sprintf("%b",index"<table>",$&)=~s|.||r/ger` (the 30-char table becomes 26 raw bytes, chr 2..29 incl. tab/newline, fine inside y///)
- `/d` deletes search chars past the replacement list: `y/A-Z,-9/../dr` turns digit lines and `.-` lines into "" (range `,-9` = 44..57 covers `,-./0-9`, 2 under `0-9.-`; `--9` dies "Ambiguous range")
- deleted chars must come LAST in the search list (mapping is positional)
