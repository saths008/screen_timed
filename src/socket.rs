use crate::csv_writer::remove_old_data;
use crate::notification::exit_with_error_notification;
use crate::{ALERT_SCREEN_ENV_VAR, SOCKET_PATH};
use std::error::Error;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fs, os::unix::net::UnixListener};

pub fn create_socket() -> Result<UnixListener, Box<dyn Error>> {
    let listener = UnixListener::bind(SOCKET_PATH)?;
    Ok(listener)
}
pub fn close_socket() -> Result<(), Box<dyn Error>> {
    fs::remove_file(SOCKET_PATH)?;
    Ok(())
}

// Send the terminating stream to close socket connection
pub fn send_terminating_mssg() {
    let mut stream = match UnixStream::connect(SOCKET_PATH) {
        Ok(stream) => stream,
        Err(err) => {
            exit_with_error_notification(format!("Error connecting to socket: {}", err).as_str());
        }
    };
    match stream.write_all(b"Terminating Stream") {
        Ok(()) => {
            println!("Terminating stream sent.");
        }
        Err(err) => {
            exit_with_error_notification(
                format!("Error sending terminating stream: {}", err).as_str(),
            );
        }
    }
    match stream.shutdown(Shutdown::Both) {
        Ok(()) => {
            println!("Stream successfully shutdown.");
        }
        Err(err) => {
            exit_with_error_notification(format!("Error shutting down stream: {}", err).as_str());
        }
    }
}
pub fn listen_for_connections(
    listener: &UnixListener,
    terminating_arc: &Arc<AtomicBool>,
    current_path: &Arc<String>,
    update_csv: &Arc<AtomicBool>,
    alert_screen_time: u64,
) -> Result<(), Box<dyn Error>> {
    for stream in listener.incoming() {
        if terminating_arc.load(Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(stream) => {
                println!("new client!");
                handle_client(stream, current_path, update_csv, alert_screen_time)?;
            }
            Err(err) => {
                println!("Error in listen_for_connections: {}", err);
                break;
            }
        }
    }
    Ok(())
}
fn handle_client(
    mut stream: UnixStream,
    current_path: &Arc<String>,
    update_csv: &Arc<AtomicBool>,
    alert_screen_time: u64,
) -> Result<(), Box<dyn Error>> {
    let mut received = String::new();
    stream.read_to_string(&mut received)?;

    if received == "UPDATE_CSV" {
        println!("Received update request!");
        update_csv.store(true, Ordering::Relaxed);
        let response = current_path.to_string();
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }
    if received == ALERT_SCREEN_ENV_VAR {
        println!("Received alert screen request!");
        stream.write_all(alert_screen_time.to_string().as_bytes())?;
        return Ok(());
    }
    if received.len() >= 7 && (received[..6].to_string() == "DELETE") {
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
        match remove_old_data(months) {
            Ok(()) => {
                println!("Successfully removed old data!");
                stream.write_all(b"Success")?;
            }
            Err(err) => {
                eprintln!("Error removing old data: {}", err);
                stream.write_all(b"Failure")?;
            }
        }
        return Ok(());
    }
    eprintln!("Received invalid request! - {}", received);
    let response = current_path.to_string();
    stream.write_all(response.as_bytes())?;
    Ok(())
}
