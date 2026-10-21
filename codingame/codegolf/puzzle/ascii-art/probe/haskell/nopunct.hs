main=interact$(\(l:_:t:r)->unlines[fromEnum<$>t>>= \d->take(read l)$drop(read l*min 26(mod(d-1)32))x|x<-r]).lines
