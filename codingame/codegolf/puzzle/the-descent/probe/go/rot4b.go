package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

const S = "00000002 00000020 00000202 00000402 00000405 00002020 00004020 00004050 00400000 00500000 00600000 00700000 00800000 05000405 05000605 05060605 08060605 20000000 23000000 23400000 23450000 23456000 23456700 23456780 23456789 50004050 50006050 50606050 80606050"

func main() {
	r := bufio.NewReader(os.Stdin)
	ok := true
	for {
		h := make([]byte, 8)
		for i := 0; i < 8; i++ {
			var l string
			if _, e := fmt.Fscan(r, &l); e != nil {
				return
			}
			h[i] = l[0]
		}
		if !strings.Contains(S, string(h)) {
			ok = false
		}
		if ok {
			m := 0
			for i := 1; i < 8; i++ {
				if h[i] > h[m] {
					m = i
				}
			}
			fmt.Println(m)
		} else {
			fmt.Println("x")
		}
	}
}
