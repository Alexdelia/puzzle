# min_by on string tokens

`gets;p gets.split.min_by{x=_1.to_i;x*x-x/2}.to_i` (temperature ruby 48, verified CG): skip `map(&:to_i)` by converting inside the key; result string -> `.to_i` also turns nil (empty list, N=0) into 0. Floor `/` makes `x*x-x/2` tie-break toward positive.
