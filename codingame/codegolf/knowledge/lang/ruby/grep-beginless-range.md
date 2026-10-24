# grep with a beginless Range = predecessors in a sorted array

- `a.grep(..v)` keeps the elements `<= v` (Range#=== on a beginless range); on a sorted array
  `a.grep(..v)[-1]` is the largest element `<= v`, `nil`/`[]` when none. 11 B vs
  `a.select{_1<=v}[-1]` 17 B (beginless `..v` also saves 2 over `0..v` where 0 is a safe lower bound).
- `.max` is the same 4 B as `[-1]`; `.last` is 5. Chained comparisons do not exist in Ruby, so to cap
  at two bounds (`<= r+l` and `<= r+S`) build the smaller bound explicitly: `[l,t].min` (9 B).
- Verified locally Ruby 3.1.2, roller-coaster: native method C 16/16,
  `q.grep(..r+[l,t].min)[-1]-r` with q the doubled prefix of the group list (scratch c1.rb of
  journal cell.roller-coaster.ruby.1792520916541526619). The idiom is 5 B under `select`.
