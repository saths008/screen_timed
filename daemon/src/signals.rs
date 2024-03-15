use std::sync::{atomic::AtomicBool, Arc};

use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;
use std::error::Error;

use crate::notification::exit_with_error_notification;

fn attempt_to_register_signals(program_finished: &Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    signal_flag::register(SIGTERM, Arc::clone(program_finished))?;
    signal_flag::register(SIGINT, Arc::clone(program_finished))?;
    signal_flag::register(SIGUSR1, Arc::clone(program_finished))?;
    signal_flag::register(SIGUSR2, Arc::clone(program_finished))?;
    Ok(())
}

pub fn register_os_signals(program_finished: &Arc<AtomicBool>) {
    if let Err(err) = attempt_to_register_signals(&program_finished) {
        exit_with_error_notification(
            format!("Exiting: Error registering signals: {}", err).as_str(),
        );
    }
}
