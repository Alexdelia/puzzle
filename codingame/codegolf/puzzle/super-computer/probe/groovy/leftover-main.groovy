e=c=0
System.in.readLines().drop(1).collect{it.split()*.toInteger()}.sort{-it[0]}.each{j,d->if(!e||j+d<=e){e=j;c++}}
print c