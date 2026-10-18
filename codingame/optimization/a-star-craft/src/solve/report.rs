use crate::simulation::{ForcedArrow, Score, Spot, Turn};

pub fn announce(
	name: &str,
	robot_count: u8,
	disk_best: Score,
	spot_list: &[Spot],
	forced_list: &[ForcedArrow],
	exhaustive: Option<u64>,
) {
	eprintln!(
		"\x1b[1;32m{name}\x1b[0m {robot_count} robots, {placeable} placeable cells, {forced} forced, disk best {disk_best}",
		placeable = spot_list.len(),
		forced = forced_list.len(),
	);
	match exhaustive {
		Some(total) => eprintln!(
			"\x1b[2mbrute force\x1b[0m \x1b[0;33m{count}\x1b[0m \x1b[2mconfigurations\x1b[0m",
			count = group_digit(total),
		),
		None => eprintln!(
			"\x1b[2msearching\x1b[0m \x1b[0;33m{space}\x1b[0m \x1b[2mconfigurations\x1b[0m",
			space = configuration_space(spot_list),
		),
	}
}

fn configuration_space(spot_list: &[Spot]) -> String {
	let mut log10 = 0.0f64;
	for spot in spot_list {
		let choice = spot.alive_count as usize + spot.removable as usize;
		log10 += (choice as f64).log10();
	}
	format_configuration_space(log10)
}

fn format_configuration_space(log10: f64) -> String {
	if log10 < 18.0 {
		let value = 10f64.powf(log10).round() as u64;
		group_digit(value)
	} else {
		let exponent = log10.floor();
		let mantissa = 10f64.powf(log10 - exponent);
		format!("{mantissa:.2}e{exponent:.0}")
	}
}

fn group_digit(value: u64) -> String {
	let raw = value.to_string();
	let mut out = String::with_capacity(raw.len() + raw.len() / 3);
	for (i, digit) in raw.char_indices() {
		if i != 0 && (raw.len() - i).is_multiple_of(3) {
			out.push(' ');
		}
		out.push(digit);
	}
	out
}

pub fn brute_progress(done: u64, total: u64, elapsed: f64, best: Score, game: Turn) {
	let rate = done as f64 / elapsed.max(1e-9);
	let eta = total.saturating_sub(done) as f64 / rate.max(1e-9);
	let percent = done as f64 / total as f64 * 100.0;
	eprint!(
		"\r\x1b[2mbrute\x1b[0m \x1b[0;1m{done}\x1b[2m/\x1b[0m\x1b[0;33m{total}\x1b[0m \x1b[0;36m{percent:>5.1}%\x1b[0m \x1b[0;96m{rate:>12.0}\x1b[2m cfg/s\x1b[0m \x1b[2meta\x1b[0m {eta} \x1b[1;32m{best}\x1b[2m best\x1b[0m \x1b[2mgame\x1b[0m {game}\x1b[K",
		done = group_digit(done),
		total = group_digit(total),
		eta = format_duration(eta as u64),
	);
}

pub fn brute_finish() {
	eprintln!();
}

fn format_duration(second: u64) -> String {
	let hour = second / 3600;
	let minute = second % 3600 / 60;
	let rest = second % 60;
	if hour > 0 {
		format!("{hour}h{minute:02}m{rest:02}s")
	} else if minute > 0 {
		format!("{minute}m{rest:02}s")
	} else {
		format!("{rest}s")
	}
}

pub struct Progress<'a> {
	pub best: Score,
	pub game_length: Turn,
	pub disk_best: Score,
	pub strategy: &'a str,
	pub mean: f64,
	pub population_max: Score,
	pub refocus: u64,
	pub nanos_per_eval: f64,
	pub moves_per_sec: f64,
	pub rounds_per_sec: f64,
	pub elapsed: u64,
	pub round: u64,
}

pub fn progress(p: &Progress) {
	let elapsed_str = if p.elapsed >= 60 {
		format!(
			"\x1b[0;34m{}\x1b[2mm \x1b[0;36m{:>2}\x1b[2ms",
			p.elapsed / 60,
			p.elapsed % 60
		)
	} else {
		format!("\x1b[0;36m{elapsed}\x1b[2ms", elapsed = p.elapsed)
	};

	eprint!(
		"\r\x1b[4A\x1b[1;32m{best:>5}\x1b[0m \x1b[2mgame\x1b[0m {game:>3} \x1b[2mdisk\x1b[0m {disk:<5} \x1b[0;35m{strategy}\x1b[0m\x1b[K
\x1b[2mpop\x1b[0m \x1b[0;36m{mean:>7.1}\x1b[2m mean\x1b[0m \x1b[0;36m{population_max:>5}\x1b[2m max\x1b[0m \x1b[0;33m{refocus}\x1b[2m refocus\x1b[0m\x1b[K
\x1b[0;38;2;52;235;198m{nanos_per_eval:>6.1}\x1b[2mns/eval \x1b[0;96m{moves_per_sec:>8.0}\x1b[2m mv/s \x1b[0;94m{rounds_per_sec:>6.0}\x1b[2m rnd/s\x1b[0m\x1b[K
{elapsed_str}\x1b[K
\x1b[0;1m{round}\x1b[0m \x1b[2mrounds\x1b[0m\x1b[K",
		best = p.best,
		game = p.game_length,
		disk = p.disk_best,
		strategy = p.strategy,
		mean = p.mean,
		population_max = p.population_max,
		refocus = p.refocus,
		nanos_per_eval = p.nanos_per_eval,
		moves_per_sec = p.moves_per_sec,
		rounds_per_sec = p.rounds_per_sec,
		round = p.round,
	);
}
