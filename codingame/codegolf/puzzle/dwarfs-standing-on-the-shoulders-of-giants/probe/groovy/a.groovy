print 2+(s=System.in.text).findAll(/\d+(?= )/).count{s=~/ $it
/}