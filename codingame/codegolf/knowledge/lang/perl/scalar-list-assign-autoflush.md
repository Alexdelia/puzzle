# `$|=(list)=...` merges autoflush into the first assignment

Verified (shadows-of-the-knight-episode-1, CG accepted 96): `$|=($L,$U)=<>=~/\d+/g;` - a list assignment in scalar context returns the RHS count (nonzero), `$|` takes any true value as 1. Saves `$|=1;` (2 bytes net).

Also verified: `for<>=~/./g` iterates the chars of the next line without `$_=<>` (`.` skips the newline, 1 under `\w`); `{...;redo}` is the cheapest endless loop; `$$_=$m[/U|D/]` picks axis from a letter with a regex index (`/U|D/` is 1 or "" -> 0).
