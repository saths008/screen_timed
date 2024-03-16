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
            let (socket, tcp_listener) = create_socket(&socket_path);
            if let Err(err) = listen_for_connections(
                &tcp_listener,
                &child_program_finished,
                &child_update_csv,
                alert_screen_time,
            ) {
                let error_message = format!("Error listening for connections: {}", err);
                exit_with_error_notification(error_message.as_str());
            }
            println!("Finished listening for connections.");

            match socket::close_socket(socket) {
                Ok(()) => {
                    println!("Socket closed!");
                }
                Err(err) => {
                    let error_message = format!("Error closing socket: {}", err);
                    exit_with_error_notification(error_message.as_str());
                }
            }
        }) {
        Ok(thread) => thread,
        Err(err) => {
            let error_message = format!("Error creating socket listener thread: {}", err);
            exit_with_error_notification(error_message.as_str());
        }
    };
    Ok(socket_listener_thread)
}
#[cfg(test)]
mod tests {

    use super::*;
    use crate::socket::{connect_to_socket, send_terminating_mssg};
    use crate::test_helpers::tests::{get_socket_path, setup};
    use serial_test::serial;
    use std::io::{Read, Write};
    use std::net::Shutdown;
    use std::path::Path;
    use std::sync::atomic::Ordering;

    #[test]
    #[serial]
    fn test_create_socket_listener_thread() {
        let (temp_dir, _) = setup();
        // let socket_path = get_socket_path(&temp_dir);

        let child_program_finished = Arc::new(AtomicBool::new(false));
        let child_update_csv = Arc::new(AtomicBool::new(false));
        let alert_screen_time = 45;
        let socket_addr = "[::1]:42345".to_string();

        //creates socket, listens for connections, and closes socket
        let socket_listener_thread = create_socket_listener_thread(
            child_program_finished.clone(),
            child_update_csv,
            alert_screen_time,
            socket_addr.clone(),
        )
        .unwrap();
        //Wait for socket_listener_thread to set up
        std::thread::sleep(std::time::Duration::from_secs(3));

        let mut stream = connect_to_socket(socket_addr.clone());
        println!("Socket connected");
        stream.write_all(b"HEALTH_CHECK").unwrap();
        println!("Sent HEALTH_CHECK request");
        println!("stream before shutdown: {:?}", stream);
        stream.shutdown(Shutdown::Write).unwrap();
        println!("stream after write shutdown: {:?}", stream);
        let mut received = String::new();
        stream.read_to_string(&mut received).unwrap();
        println!("response received from socket: {}", received);

        //terminate socket_listener_thread
        child_program_finished.store(true, Ordering::Relaxed);
        send_terminating_mssg(socket_addr.clone());

        println!("program_finished set to true");
        assert_eq!(received, "Ok");
        socket_listener_thread.join().unwrap();

        //if socket is closed, it does not exist
    }
}
