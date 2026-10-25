cat >/tmp/thx1 <<'E'
read a b c d
s=WEN;h=${s:a%3:1}
yes S$h|sed $[b-d]q;yes $h
E
. /tmp/thx1
