# Growing work list in Ruby

`k.map{|i|...k<<a<<b}` iterates elements appended during the loop (like `each`, 1 byte shorter, result array discarded). `(a[i]=k)<<i-1<<i+1<<i-w<<i+w` beats `.push(..)` by 2 bytes (`<<` binds looser than `+ -`). Verified surface ruby 146 (3.1.2), 360k-cell flood fine. Mark cells with the list itself, answer `a[s].size/4` (land `"#"`.size=1 -> 0).
