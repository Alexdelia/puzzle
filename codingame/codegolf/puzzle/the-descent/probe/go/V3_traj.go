package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

const T = "06708501 06700501 06400501 02400501 02400301 02000301 02000001 00000001"

func main() {
	r := bufio.NewReader(os.Stdin)
	ok := true
	t := 0
	g := false
	for {
		h := make([]byte, 8)
		for i := 0; i < 8; i++ {
			var l string
			if _, e := fmt.Fscan(r, &l); e != nil {
				return
			}
			h[i] = l[0]
		}
		if t == 0 {
			g = string(h) == T[:8]
		}
		if g && !strings.Contains(T, string(h)) {
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
