package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

const S = "00002000 00202000 00400000 00402000 00402050 00405050 00500000 00600000 00605050 00700000 00800000 20000000 23000000 23400000 23450000 23456000 23456700 23456780 23456789 60605050 60605080"

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
