# string-key argmax (the-descent 58 B)

`for(;;print(m[1]))for(m=i=0;i<8;m=m<h?h:m)h=readline()+i++`

- key = line+index as string, max by string compare, answer `m[1]`; m=0 numeric first compare coerces, fine
- running max in the for update clause beats `(h=..)>m&&(m=h)` by 2 B
- numeric `readline()*8+i++` then `m%8` is 61
- verified local 15/15 and CG
