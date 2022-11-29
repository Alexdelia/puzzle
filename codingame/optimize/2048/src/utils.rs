#[macro_export]
macro_rules! err {
	($($arg:tt)*) => {
		eprint!("\x1b[31;1merror\x1b[0m\x1b[1m:\t");
		eprint!($($arg)*);
		eprintln!("\x1b[0m");
	};
}
