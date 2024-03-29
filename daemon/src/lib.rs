use config::new_config;
use csv_writer::write_data_to_csv;
use notification::{exit_with_error_notification, screen_time_notification};
use screen_time::update_current_app;
use signals::register_os_signals;
use socket::send_terminating_mssg;
use std::collections::HashMap;
use std::error::Error;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;
use std::time::{self};
use threads::{create_alert_screen_thread, create_socket_listener_thread};

mod config;
mod csv_writer;
mod notification;
mod screen_time;
mod signals;
mod socket;
mod test_helpers;
mod threads;

#[cfg(target_os = "windows")]
mod windows;

const SOCKET_ADDR: &str = "[::1]:12345";
const ALERT_SCREEN_ENV_VAR: &str = "ALERT_SCREEN";
const SCREEN_DATA_CSV_PATH: &str = "screen_time_data.csv";

pub fn run() -> Result<(), Box<dyn Error>> {
    let env_config = new_config();
    let alert_screen_time = env_config.get_alert_screen_time();

    if let Err(err) = create_alert_screen_thread(alert_screen_time) {
        exit_with_error_notification(
            format!("Error creating alert screen thread: {}", err).as_str(),
        );
    }
    // When true, update csv
    let update_csv = Arc::new(AtomicBool::new(false));
    let child_update_csv = Arc::clone(&update_csv);
    let program_finished = Arc::new(AtomicBool::new(false));
    register_os_signals(&program_finished);

    let child_program_finished = Arc::clone(&program_finished);

    let socket_addr: &str = "[::1]:12345";

    let socket_listener_thread = match create_socket_listener_thread(
        Arc::clone(&child_program_finished),
        Arc::clone(&child_update_csv),
        alert_screen_time,
        socket_addr.to_string(),
    ) {
        Ok(socket_listener_thread) => socket_listener_thread,
        Err(err) => {
            exit_with_error_notification(
                format!("Error creating socket screen thread: {}", err).as_str(),
            );
        }
    };
    let mut program_times: HashMap<String, time::Duration> = HashMap::new();

    // 1, 0 ->  1 - run screen_time_daemon
    // 0, 1 ->  1 -  break
    // 0, 0 -> 0 - break
    // 1, 1 -> 1 - update
    while !program_finished.load(Ordering::Relaxed) || update_csv.load(Ordering::Relaxed) {
        if program_finished.load(Ordering::Relaxed) {
            break;
        }
        if update_csv.load(Ordering::Relaxed) {
            println!("Updating csv...");
            if let Err(err) = write_data_to_csv(
                &program_times,
                &SCREEN_DATA_CSV_PATH.to_string(),
                SystemTime::now(),
            ) {
                exit_with_error_notification(format!("Error writing to csv: {}", err).as_str());
            }
            program_times.clear();
            update_csv.store(false, Ordering::Relaxed);
        }

        thread::sleep(time::Duration::from_secs(1));
        update_current_app(&mut program_times);
    }

    println!("Signal received!");
    send_terminating_mssg(SOCKET_ADDR.to_string());

    for (program_name, duration) in &program_times {
        println!("{}: {}", program_name, duration.as_secs());
    }
    match write_data_to_csv(
        &program_times,
        &SCREEN_DATA_CSV_PATH.to_string(),
        SystemTime::now(),
    ) {
        Ok(()) => {
            println!("Finished writing to csv.");
        }
        Err(err) => {
            exit_with_error_notification(format!("Error writing to csv: {}", err).as_str());
        }
    }
    //Wait for socket listener thread to finish
    if let Err(_) = socket_listener_thread.join() {
        exit_with_error_notification("Error joining socket listener thread");
    }
    println!("Successfully exiting...");

    Ok(())
}
