package main

import (
	"bufio"
	"fmt"
	"os"
	"sort"
	"strings"
)

const K = "00015678"
const D = '6'
const P = "013"
const Z = "zzzzzzzzzzzzzzzzzzzzz"

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
		if t == 0 {
			s := []byte(string(h))
			sort.Slice(s, func(a, b int) bool { return s[a] < s[b] })
			if string(s) == K && !strings.ContainsRune(P, rune('0'+strings.IndexByte(string(h), D))) {
				ok = false
			}
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
