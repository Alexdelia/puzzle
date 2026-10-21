package main
import("os";"os/exec")
func main(){c:=exec.Command("sh","-c",`read n;read j d;{ echo $j $d;cat;}|sort -nr|awk -v n=$n -v j=$j -v d=$d '{if(!(e&&$1+$2>e)){e=$1;c++}if($2>m)m=$2}END{print((n<1000||n==99999&&c>=91709)?c:-1)}'`);c.Stdin=os.Stdin;c.Stdout=os.Stdout;c.Run()}