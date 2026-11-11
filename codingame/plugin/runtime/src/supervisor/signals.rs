use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result};
use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, sigaction};

const STOP_SIGNALS: [Signal; 3] = [Signal::SIGINT, Signal::SIGTERM, Signal::SIGHUP];

static RECEIVED: AtomicU32 = AtomicU32::new(0);

extern "C" fn count_stop_signal(_: libc::c_int) {
	RECEIVED.fetch_add(1, Ordering::SeqCst);
}

pub(super) fn install_stop_signals() -> Result<()> {
	let action = SigAction::new(
		SigHandler::Handler(count_stop_signal),
		SaFlags::SA_RESTART,
		SigSet::empty(),
	);
	for signal in STOP_SIGNALS {
		unsafe { sigaction(signal, &action) }.with_context(|| format!("handle {signal}"))?;
	}
	Ok(())
}

pub(super) fn stop_signals_received() -> u32 {
	RECEIVED.load(Ordering::SeqCst)
}
