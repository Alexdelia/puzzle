readline();for(var s='',t;t=readline();s=t.match(s)?t:s.match(t)?s:(s+`
`+t).replace(/(.*)\n\1/,'$1'));console.log(s.length)