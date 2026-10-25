# String literal is not indexable

`"WEN"[1]` is a syntax error in Perl 5.34.0 and 5.42.3 (verified). A string literal cannot
be used directly as an array/list or with `[]` indexing:

```
perl -e 'print "WEN"[1]'      # syntax error near ""WEN"["
perl -e '$s="WEN";print $s[1]'  # E: a scalar variable is indexed as a string
perl -e 'print (W,E,N)[1]'     # E: parenthesised bareword list
```

Consequences for golf: to pick a character by position, use a parenthesised **list**
`(W,E,N)[$i]` (13 B for a 3-element letter table with a 4 B index), a stored string
`$s="WEN";$s[$i]` (costs the store), `substr` (17 B), or build the letter arithmetically
with `chr` (usually longer). Do not count on `"ABC"[$i]` saving the brackets.
