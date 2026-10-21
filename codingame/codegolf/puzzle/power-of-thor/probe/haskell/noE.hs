main=interact$(\(a:b:c:d:_)->unlines[['S'|i<b-d]++['N'|i<d-b]++['W'|i<c-a]|i<-[0..]]).map read.words
