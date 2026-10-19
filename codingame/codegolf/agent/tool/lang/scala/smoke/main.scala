object Solution extends App {
  val Array(a, b) = scala.io.StdIn.readLine().trim.split(" ").map(_.toInt)
  println(a + b)
}
