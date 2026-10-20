# for over a growing array as work list

`for(@{$k=[$start]}){$g[$_]=~O&&($g[$_]=$k,push@$k,...)}` iterates elements pushed during the loop (BFS without queue). `for(my@k=...)` iterates a copy and does not grow. Mark cells with the ref itself, answer `@{$g[$s]}>>2` (`>>2` is 1 byte under `/4|0`); a non-ref string like `#` used as `@{"#"}` is an empty array (no strict). `=~O` (bareword pattern) is 1 byte under `eq O`; `/(.+) (.+)/g` is 4 under `(\d+) (\d+)` when rows have no space. Verified surface 17/17.
