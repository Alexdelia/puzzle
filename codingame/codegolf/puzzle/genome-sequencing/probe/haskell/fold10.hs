import Data.List
main=interact$show.length.foldl g"".tail.words
g s t=(if isInfixOf s t then""else s)++drop(last[k|k<-[0..10],isInfixOf t$s++drop k t])t
