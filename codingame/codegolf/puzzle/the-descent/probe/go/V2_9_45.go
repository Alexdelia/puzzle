package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

const Z = "zzzzzzzzz"

func main() {
	r := bufio.NewReader(os.Stdin)
	ok := true
	t := 0
	for {
		h := make([]byte, 8)
		for i := 0; i < 8; i++ {
			var l string
			if _, e := fmt.Fscan(r, &l); e != nil {
				return
			}
			h[i] = l[0]
		}
		s := string(h)
		if t == 0 && strings.Contains(s, "9") && s != "23456789" && !strings.Contains("45", string(rune('0'+strings.IndexByte(s, '9')))) {
			ok = false
		}
		t++
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
