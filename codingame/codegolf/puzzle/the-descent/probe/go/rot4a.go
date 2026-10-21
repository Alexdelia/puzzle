package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

const S = "00000002 00000020 00200000 00400000 00500000 00600000 00700000 00800000 02000002 02000004 02050004 05050004 05050006 05050606 05080606 20000000 20000020 20000040 20200000 20500040 23000000 23400000 23450000 23456000 23456700 23456780 23456789 40200000 40205000 40505000 40505060 50500040 50500060 50506060 50806060 60505060 60508060"

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
