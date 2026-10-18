use mpr::bot_search::{Budget, SearchBot};
use mpr::cgio::run_cg_bot;

fn main() {
	run_cg_bot(SearchBot::new(Budget::TimeMs(65), 0x5EED));
}
