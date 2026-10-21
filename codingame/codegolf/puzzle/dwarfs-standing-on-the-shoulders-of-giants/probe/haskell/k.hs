main=interact$show.(+2).length.f.map words.lines
f(_:w)=[a|a:_<-w,elem a$last<$>w]