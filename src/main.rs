use active_win_pos_rs::get_active_window;
use std::collections::HashMap;
use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

fn screen_time_daemon(program_times: &mut HashMap<String, time::Duration>) {
    match get_active_window() {
        Ok(active_window) => {
            let app_name = &active_window.app_name;
            if program_times.contains_key(app_name) {
                let existing_time = program_times.get(app_name).unwrap();
                let new_time = *existing_time + time::Duration::from_secs(1);
                program_times.insert(app_name.to_string(), new_time);
            } else {
                program_times.insert(app_name.to_string(), time::Duration::from_secs(1));
            }
            // println!("app name: {}", app_name);
            // println!("active window: {:#?}", active_window);
        }
        Err(()) => {
            println!("error occurred while getting the active window");
        }
    }
}

fn main() -> Result<(), Error> {
    let mut program_times: HashMap<String, time::Duration> = HashMap::new();
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;
    // Keep executing as long the SIGTERM has not been called.
    while !term.load(Ordering::Relaxed) {
        thread::sleep(time::Duration::from_secs(1));
        screen_time_daemon(&mut program_times);
    }

    println!("Sigint received!");
    for (program_name, duration) in &program_times {
        println!("{}: {}", program_name, duration.as_secs());
    }
    Ok(())
}
