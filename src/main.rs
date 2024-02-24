use active_win_pos_rs::get_active_window;
use csv::ReaderBuilder;
use csv::WriterBuilder;
use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::TimestampSeconds;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::net::Shutdown;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use std::time::{self, SystemTime};
fn close_socket() -> Result<(), Box<dyn Error>> {
    std::fs::remove_file("/tmp/screen-time-sock")?;
    Ok(())
}

fn listen_for_connections(
    listener: &UnixListener,
    terminating_arc: &Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    for stream in listener.incoming() {
        if terminating_arc.load(Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(stream) => {
                println!("new client!");
                handle_client(stream)?;
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
fn handle_client(mut stream: UnixStream) -> Result<(), Box<dyn Error>> {
    let mut received = String::new();
    stream.read_to_string(&mut received)?;
    println!("{}", received);

    let response = "Hello from the server!";
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn read_csv(csv_path: String) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().from_path(csv_path)?;
    for result in rdr.records() {
        let record = result?;
        println!("{:?}", record);
    }
    println!("Finish read_csv method");
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
fn _write_data_to_csv(
    _program_times: &HashMap<String, time::Duration>,
    csv_path: String,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new().has_headers(true).from_path(csv_path)?;
    for (program_name, duration) in _program_times {
        wtr.serialize(Row {
            timestamp: SystemTime::now(),
            application: program_name,
            duration: duration.as_secs(),
        })?;
    }
    Ok(())
}
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
        }
        Err(()) => {
            println!("error occurred while getting the active window");
        }
    }
}

fn main() -> Result<(), io::Error> {
    let pid = std::process::id();
    println!("PID of the current Rust program: {}", pid);
    let program_finished = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&program_finished))?;
    let child_program_finished = Arc::clone(&program_finished);

    thread::Builder::new()
        .name("screen_time_daemon".to_string())
        .spawn(move || {
            let listener = create_socket().unwrap();
            listen_for_connections(&listener, &child_program_finished).unwrap();

            println!("Closing socket...");
            close_socket().unwrap();
            println!("Socket closed!");
        })?;

    let mut program_times: HashMap<String, time::Duration> = HashMap::new();
    //
    // Keep executing as long the SIGTERM has not been called.
    while !program_finished.load(Ordering::Relaxed) {
        thread::sleep(time::Duration::from_secs(1));
        screen_time_daemon(&mut program_times);
    }

    println!("Sigint received!");
    let mut stream = UnixStream::connect("/tmp/screen-time-sock")?;
    stream.write_all(b"Terminating Stream")?;
    println!("Terminating stream sent!");
    stream
        .shutdown(Shutdown::Both)
        .expect("shutdown function failed");

    for (program_name, duration) in &program_times {
        println!("{}: {}", program_name, duration.as_secs());
    }
    let read_result = read_csv("screen_time_data.csv".to_string());
    match read_result {
        Ok(_) => println!("read csv success"),
        Err(e) => println!("error reading csv: {}", e),
    }
    Ok(())
}
