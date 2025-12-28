use std::collections::HashMap;

fn main() {
	let n = {
		let mut input_line = String::new();
		std::io::stdin().read_line(&mut input_line).unwrap();
		input_line.parse::<usize>().unwrap()
	};

	dbg!(n);
	println!("{}", fraction_to_decimal(n));
}

fn fraction_to_decimal(d: usize) -> String {
	let mut res = String::new();

	let n = 1usize;

	res.push_str((n / d).to_string().as_str());

	let mut r = n % d;
	if r == 0 {
		return res;
	}

	res.push('.');

	let mut memo = HashMap::<usize, usize>::new();
	let mut digit_list = Vec::<u8>::new();
	let mut repeat_index: Option<usize> = None;

	while r != 0 {
		if let Some(&pos) = memo.get(&r) {
			repeat_index = Some(pos);
			break;
		}

		memo.insert(r, digit_list.len());

		r *= 10;
		let digit = r / d;
		digit_list.push(digit as u8);
		r %= d;
	}

	if let Some(i) = repeat_index {
		for &digit in &digit_list[0..i] {
			res.push((b'0' + digit) as char);
		}

		res.push('(');
		for &digit in &digit_list[i..] {
			res.push((b'0' + digit) as char);
		}
		res.push(')');
	} else {
		for &digit in &digit_list {
			res.push((b'0' + digit) as char);
		}
	}

	res
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_example() {
		for &(d, expected) in &[
			(4, "0.25"),
			(7, "0.(142857)"),
			(20, "0.05"),
			(25000, "0.00004"),
			(195312500, "0.00000000512"),
			(44, "0.02(27)"),
			(561, "0.(0017825311942959)"),
			(
				1931,
				"0.(00051786639047125841532884515794924909373381667529777317452097358881408596582081822889694458829621957534955981356809943034697048161574313827032625582599689280165717244950802692905230450543759709994821336095287415846711548420507509062661833247022268254790264111859140341791817711030554117037804246504401864319005696530295183842568617296737441740031071983428275504919730709476954945624029)",
			),
			(210, "0.0(047619)"),
			(
				7389400,
				"0.000(0001353289847619563158037188405012585595582861937369745852166617046038920616017538636425149538528161961728963109318753890708311906244079356916664411183587300728069938019324979024007361896771050423579722304923268465639970768939291417435786396730451728151135410182152813489593201071805559314694021165453216769967791701626654396838714915960700462825127885890600048718434514304273689338782580453081440983029745310850677998213657401142176631390911305383387013830622242671935475140065499228624786856848999918802609142826210517768695699244864265028283757815248870002977237664763038947681814491027688310282296262213440874766557501285625355238585)",
			),
		] {
			let found = fraction_to_decimal(d);
			assert_eq!(
				found, expected,
				"1/{d} should be '{expected}', but found '{found}'"
			);
		}
	}
}
