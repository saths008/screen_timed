use active_win_pos_rs::get_active_window;
use csv::WriterBuilder;
use notify_rust::Notification;
use notify_rust::Timeout;
use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::TimestampSeconds;
use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::net::Shutdown;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{self, SystemTime};

const SOCKET_PATH: &str = "/tmp/screen-time-sock";

fn close_socket() -> Result<(), Box<dyn Error>> {
    std::fs::remove_file(SOCKET_PATH)?;
    Ok(())
}

fn listen_for_connections(
    listener: &UnixListener,
    terminating_arc: &Arc<AtomicBool>,
    current_path: &Arc<String>,
    update_csv: &Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    for stream in listener.incoming() {
        if terminating_arc.load(Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(stream) => {
                println!("new client!");
                handle_client(stream, current_path, update_csv)?;
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
    Ok(())
}
fn create_socket() -> Result<UnixListener, Box<dyn Error>> {
    let listener = UnixListener::bind("/tmp/screen-time-sock")?;
    Ok(listener)
}
fn handle_client(
    mut stream: UnixStream,
    current_path: &Arc<String>,
    update_csv: &Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    let mut received = String::new();
    stream.read_to_string(&mut received)?;

    if received == "update" {
        println!("Received update request!");
        update_csv.store(true, Ordering::Relaxed);
    } else {
        println!("Received invalid request! - {}", received);
    }

    let response = current_path.to_string();
    stream.write_all(response.as_bytes())?;
    Ok(())
}

#[serde_as]
#[derive(Deserialize, Serialize)]
struct Row<'a> {
    #[serde_as(as = "TimestampSeconds<i64>")]
    timestamp: SystemTime,
    application: &'a str,
    duration: u64,
}
fn write_data_to_csv(
    program_times: &HashMap<String, time::Duration>,
    csv_path: &String,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new().append(true).open(csv_path)?;
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);
    for (program_name, duration) in program_times {
        wtr.serialize(Row {
            timestamp: SystemTime::now(),
            application: program_name,
            duration: duration.as_secs(),
        })?;
    }
    wtr.flush()?;
    Ok(())
}
fn error_notification(error_message: &str) {
    eprintln!("{}", &error_message);
    Notification::new()
        .summary("Error")
        .body(error_message)
        .timeout(Timeout::Never)
        .show()
        .unwrap();
    exit(1);
}
fn screen_time_daemon(program_times: &mut HashMap<String, time::Duration>) {
    match get_active_window() {
        Ok(active_window) => {
            let app_name = &active_window.app_name;
            if program_times.contains_key(app_name) {
                let existing_time = match program_times.get(app_name) {
                    Some(time) => time,
                    None => {
                        error_notification("Error getting existing time");
                        return;
                    }
                };
                let new_time = *existing_time + time::Duration::from_secs(1);
                program_times.insert(app_name.to_string(), new_time);
            } else {
                program_times.insert(app_name.to_string(), time::Duration::from_secs(1));
            }
        }
        Err(()) => {
            //Could happen when switching windows.
            println!("error occurred while getting the active window");
        }
    }
}

fn register_signals(program_finished: &Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    signal_flag::register(SIGTERM, Arc::clone(program_finished))?;
    signal_flag::register(SIGINT, Arc::clone(program_finished))?;
    signal_flag::register(SIGUSR1, Arc::clone(program_finished))?;
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let pid = std::process::id();
    println!("PID of the current Rust program: {}", pid);
    // When true, update csv
    let update_csv = Arc::new(AtomicBool::new(false));
    let child_update_csv = Arc::clone(&update_csv);

    let program_finished = Arc::new(AtomicBool::new(false));
    if let Err(err) = register_signals(&program_finished) {
        error_notification(format!("Exiting: Error registering signals: {}", err).as_str());
        exit(1);
    }
    let child_program_finished = Arc::clone(&program_finished);
    let current_path: PathBuf = match std::env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            error_notification(format!("Exiting: Error getting current path: {}", err).as_str());
            exit(1);
        }
    };
    let current_path_str = match current_path.to_str() {
        Some(path) => {
            let mut full_path = path.to_string();
            full_path.push_str("/screen_time_data.csv");
            Arc::new(full_path)
        }
        None => {
            println!("Error getting current path");
            exit(1);
        }
    };

    if let Err(err) = thread::Builder::new()
        .name("socket_listener_thread".to_string())
        .spawn(move || {
            let listener = match create_socket() {
                Ok(listener) => listener,
                Err(err) => {
                    error_notification(format!("Error creating socket: {}", err).as_str());
                    exit(1);
                }
            };
            if let Err(err) = listen_for_connections(
                &listener,
                &child_program_finished,
                &current_path_str,
                &child_update_csv,
            ) {
                error_notification(format!("Error listening for connections: {}", err).as_str());
                exit(1);
            }
            println!("Finished listening for connections.");

            match close_socket() {
                Ok(()) => {
                    println!("Socket closed!");
                }
                Err(err) => {
                    error_notification(format!("Error closing socket: {}", err).as_str());
                }
            }
        })
    {
        error_notification(format!("Error creating socket listener thread: {}", err).as_str());
        exit(1);
    }

    let mut program_times: HashMap<String, time::Duration> = HashMap::new();

    let csv_path = "screen_time_data.csv".to_string();
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
            if let Err(err) = write_data_to_csv(&program_times, &csv_path) {
                error_notification(format!("Error writing to csv: {}", err).as_str());
            }
            program_times.clear();
            update_csv.store(false, Ordering::Relaxed);
        }

        thread::sleep(time::Duration::from_secs(1));
        screen_time_daemon(&mut program_times);
    }

    println!("Signal received!");
    let mut stream = match UnixStream::connect(SOCKET_PATH) {
        Ok(stream) => stream,
        Err(err) => {
            error_notification(format!("Error connecting to socket: {}", err).as_str());
            exit(1);
        }
    };
    match stream.write_all(b"Terminating Stream") {
        Ok(()) => {
            println!("Terminating stream sent.");
        }
        Err(err) => {
            error_notification(format!("Error sending terminating stream: {}", err).as_str());
        }
    }
    match stream.shutdown(Shutdown::Both) {
        Ok(()) => {
            println!("Stream successfully shutdown.");
        }
        Err(err) => {
            error_notification(format!("Error shutting down stream: {}", err).as_str());
        }
    }

    for (program_name, duration) in &program_times {
        println!("{}: {}", program_name, duration.as_secs());
    }
    match write_data_to_csv(&program_times, &csv_path) {
        Ok(()) => {
            println!("Finished writing to csv.");
        }
        Err(err) => {
            error_notification(format!("Error writing to csv: {}", err).as_str());
            exit(1);
        }
    }
    Ok(())
}
