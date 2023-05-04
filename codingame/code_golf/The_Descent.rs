fn main(){loop{let mut t=(0,0);for i in 0..8{let n=&mut"".into();std::io::stdin().read_line(n);let x=n.as_bytes()[0];if x>t.0{t=(x,i)}}print!("{}
",t.1);}}
