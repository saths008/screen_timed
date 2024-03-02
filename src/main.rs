use active_win_pos_rs::get_active_window;
use csv::ReaderBuilder;
use csv::WriterBuilder;
use notify_rust::Notification;
use notify_rust::Timeout;
use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::TimestampSeconds;
use signal_hook::consts::signal::*;
use signal_hook::flag as signal_flag;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io;
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
use std::thread::JoinHandle;
use std::time::UNIX_EPOCH;
use std::time::{self, SystemTime};

const SOCKET_PATH: &str = "/tmp/screen-time-sock";
const ALERT_SCREEN_ENV_VAR: &str = "ALERT_SCREEN";
const SCREEN_DATA_CSV_PATH: &str = "screen_time_data.csv";

fn remove_old_data(months: u32) -> Result<(), Box<dyn Error>> {
    let backup_screen_csv_path = format!("backup_{}", SCREEN_DATA_CSV_PATH);
    fs::copy(SCREEN_DATA_CSV_PATH, &backup_screen_csv_path)?;
    let new_screen_csv_path = format!("new_{}", SCREEN_DATA_CSV_PATH);

    fs::File::create(&new_screen_csv_path)?;
    let mut rdr = ReaderBuilder::new().from_path(&backup_screen_csv_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_path(&new_screen_csv_path)?;

    let mut rdr_iter = rdr.deserialize();
    let first_result_iter = rdr_iter.next();
    if first_result_iter.is_none() {
        return Ok(());
    }
    let first_result: Row = first_result_iter.unwrap()?;
    let first_timestamp = first_result.timestamp.duration_since(UNIX_EPOCH)?;
    // 30 days approximation in a month
    let end_timestamp =
        first_timestamp + time::Duration::from_secs((60 * 60 * 24 * 30 * months).into());
    for result in rdr_iter {
        let record: Row = result?;
        let timestamp = record.timestamp.duration_since(UNIX_EPOCH)?;
        let should_delete = timestamp >= first_timestamp && timestamp <= end_timestamp;
        if !should_delete {
            wtr.serialize(record)?;
        }
    }
    wtr.flush()?;
    //replace old csv with new csv
    fs::rename(new_screen_csv_path, SCREEN_DATA_CSV_PATH)?;
    fs::remove_file(backup_screen_csv_path)?;
    println!("Successfully removed {} months old data", months);
    Ok(())
}
fn close_socket() -> Result<(), Box<dyn Error>> {
    fs::remove_file(SOCKET_PATH)?;
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
                println!("Error in listen_for_connections: {}", err);
                break;
            }
        }
    }
    Ok(())
}
fn create_socket() -> Result<UnixListener, Box<dyn Error>> {
    let listener = UnixListener::bind(SOCKET_PATH)?;
    Ok(listener)
}
fn handle_client(
    mut stream: UnixStream,
    current_path: &Arc<String>,
    update_csv: &Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    let mut received = String::new();
    stream.read_to_string(&mut received)?;

    if received == "UPDATE_CSV" {
        println!("Received update request!");
        update_csv.store(true, Ordering::Relaxed);
    } else if received == ALERT_SCREEN_ENV_VAR {
        println!("Received alert screen request!");
        let alert_screen_time = get_env_var(ALERT_SCREEN_ENV_VAR)?;
        stream.write_all(alert_screen_time.as_bytes())?;
        return Ok(());
    } else if received.len() >= 7 && (received[..6].to_string() == "DELETE") {
        println!("Received delete request!");
        let months_str = received[7..].trim().to_string();
        let months: u32 = match months_str.parse() {
            Ok(months) => months,
            Err(err) => {
                eprintln!("Error parsing months: {}", err);
                stream.write_all(b"Failure")?;
                return Ok(());
            }
        };
        if let Err(err) = remove_old_data(months) {
            eprintln!("Error removing old data: {}", err);
            stream.write_all(b"Failure")?;
        } else {
            stream.write_all(b"Success")?;
            println!("Successfully removed old data!");
        }
        return Ok(());
    } else {
        eprintln!("Received invalid request! - {}", received);
    }

    let response = current_path.to_string();
    stream.write_all(response.as_bytes())?;
    Ok(())
}

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Row {
    #[serde_as(as = "TimestampSeconds<i64>")]
    timestamp: SystemTime,
    application: String,
    //How long in seconds the application was active
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
            application: program_name.to_string(),
            duration: duration.as_secs(),
        })?;
    }
    wtr.flush()?;
    Ok(())
}
fn error_notification(error_message: &str) {
    eprintln!("{}", &error_message);
    if let Err(err) = Notification::new()
        .summary("Error")
        .body(error_message)
        .timeout(Timeout::Never)
        .show()
    {
        eprintln!("Error showing notification: {}", err);
    }

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
    signal_flag::register(SIGUSR2, Arc::clone(program_finished))?;
    Ok(())
}
fn screen_time_notification(alert_screen_time: u64) {
    let alert_message = format!(
        "You have been on the screen for {} minutes",
        alert_screen_time
    );
    if let Err(err) = Notification::new()
        .summary("Screen Time Alert")
        .body(alert_message.as_str())
        .timeout(Timeout::Never)
        .show()
    {
        eprintln!("Error showing notification: {}", err);
        exit(1);
    }
}
fn create_alert_screen_thread(alert_screen_time: u64) -> io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("alert_screen_thread".to_string())
        .spawn(move || {
            if alert_screen_time == 0 {
                return;
            }
            loop {
                thread::sleep(time::Duration::from_secs(alert_screen_time * 60));
                screen_time_notification(alert_screen_time);
            }
        })
}
fn get_env_var(env_var: &str) -> Result<String, Box<dyn Error>> {
    let alert_screen_env_str: String = dotenvy::var(env_var)?;
    Ok(alert_screen_env_str.trim().to_string())
}
fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    let alert_screen_time_str = match get_env_var(ALERT_SCREEN_ENV_VAR) {
        Ok(alert_screen_env) => alert_screen_env,
        Err(err) => {
            error_notification(format!("Error getting ALERT_SCREEN_ENV_VAR: {}", err).as_str());
            exit(1);
        }
    };
    let alert_screen_time: u64 = match alert_screen_time_str.parse() {
        Ok(alert_screen_env) => alert_screen_env,
        Err(err) => {
            error_notification(format!("Error getting ALERT_SCREEN_ENV_VAR: {}", err).as_str());
            exit(1);
        }
    };

    if let Err(err) = create_alert_screen_thread(alert_screen_time) {
        error_notification(format!("Error creating alert screen thread: {}", err).as_str());
        exit(1);
    }
    // When true, update csv
    let update_csv = Arc::new(AtomicBool::new(false));
    let child_update_csv = Arc::clone(&update_csv);

    let program_finished = Arc::new(AtomicBool::new(false));
    if let Err(err) = register_signals(&program_finished) {
        error_notification(format!("Exiting: Error registering signals: {}", err).as_str());
        exit(1);
    }
    let child_program_finished = Arc::clone(&program_finished);
    let current_path: PathBuf = match env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            error_notification(format!("Exiting: Error getting current path: {}", err).as_str());
            exit(1);
        }
    };
    let current_path_str = match current_path.to_str() {
        Some(path) => {
            let mut full_path = path.to_string();
            full_path.push_str(format!("/{}", SCREEN_DATA_CSV_PATH.to_string()).as_str());
            Arc::new(full_path)
        }
        None => {
            println!("Error getting current path");
            exit(1);
        }
    };

    let socket_listener_thread = match thread::Builder::new()
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
        }) {
        Ok(thread) => thread,
        Err(err) => {
            error_notification(format!("Error creating socket listener thread: {}", err).as_str());
            exit(1);
        }
    };

    let mut program_times: HashMap<String, time::Duration> = HashMap::new();

    let csv_path = SCREEN_DATA_CSV_PATH.to_string();
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
    //Wait for socket listener thread to finish
    if let Err(_) = socket_listener_thread.join() {
        error_notification("Error joining socket listener thread");
        exit(1);
    }
    println!("Successfully exiting...");

    Ok(())
}
