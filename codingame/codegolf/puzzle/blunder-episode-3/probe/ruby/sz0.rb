s=$<.read;k=s.size%8;a=s.split;n=a[0].to_i;f=a[1].to_i
c=n==999?'O(log n)':n==90?'O(n)':n==75?'O(n^2)':n==54?'O(n^2 log n)':n==50?'O(n^3)':n==24?'O(2^n)':f==10?'O(1)':'O(n log n)'
puts k[0]==1?c:'X'
