# Scanner + unzip + doubled median (network-cabling, 215 bytes, CG submitted)

- `val s=new java.util.Scanner(System.in);val(x,y)=Seq.fill(s.nextInt)(s.nextLong->s.nextLong).unzip` reads `N` then `N` pairs into two Lists, 15 shorter than `scala.io.Source.stdin.mkString.split("\\s").tail.map(_.toLong).grouped(2).toArray.transpose`
- `val(l,h)=(y++y).sorted.splitAt(y.size);(h.sum-l.sum)/2` = sum |y-median|, no odd case, no index
- `List(i)` is O(n): `u(u.size/2)` on a List with 100000 items times out (6 s) when evaluated per element; index an Array or once
- `print` instead of `println` accepted
- verified locally 21/21 (not a separate CG proof beyond submit.sh confirm)
