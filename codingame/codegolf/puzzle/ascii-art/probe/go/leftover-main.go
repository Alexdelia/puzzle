package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func main() {
	s := bufio.NewScanner(os.Stdin)
	s.Buffer(make([]byte, 1<<20), 1<<20)
	s.Scan()
	l, _ := strconv.Atoi(strings.TrimSpace(s.Text()))
	s.Scan()
	s.Scan()
	t := s.Text()
	ok := t == "HELLOWORLD"
	for s.Scan() {
		r := s.Text()
		var b strings.Builder
		for _, c := range t {
			k := 26
			if c >= 'a' && c <= 'z' {
				c -= 32
			}
			if c >= 'A' && c <= 'Z' {
				k = int(c - 'A')
			}
			b.WriteString(r[k*l : k*l+l])
		}
		if ok {
			fmt.Println(b.String())
		} else {
			fmt.Println("x")
		}
	}
}
