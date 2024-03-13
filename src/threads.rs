use crate::socket::{create_socket, listen_for_connections};
use crate::{notification::exit_with_error_notification, screen_time_notification, socket};
use std::{
    error::Error,
    io,
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
    time,
};

pub fn create_alert_screen_thread(alert_screen_time: u64) -> io::Result<JoinHandle<()>> {
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
pub fn create_socket_listener_thread(
    child_program_finished: Arc<AtomicBool>,
    child_update_csv: Arc<AtomicBool>,
    alert_screen_time: u64,
    socket_path: String,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    let socket_listener_thread = match thread::Builder::new()
        .name("socket_listener_thread".to_string())
        .spawn(move || {
            let listener = create_socket(&socket_path);
            if let Err(err) = listen_for_connections(
                &listener,
                &child_program_finished,
                &child_update_csv,
                alert_screen_time,
            ) {
                exit_with_error_notification(
                    format!("Error listening for connections: {}", err).as_str(),
                );
            }
            println!("Finished listening for connections.");

            match socket::close_socket(&socket_path) {
                Ok(()) => {
                    println!("Socket closed!");
                }
                Err(err) => {
                    exit_with_error_notification(format!("Error closing socket: {}", err).as_str());
                }
            }
        }) {
        Ok(thread) => thread,
        Err(err) => {
            exit_with_error_notification(
                format!("Error creating socket listener thread: {}", err).as_str(),
            );
        }
    };
    Ok(socket_listener_thread)
}
