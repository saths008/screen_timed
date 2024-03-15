use crate::csv_writer::{get_curr_path_to_csv, remove_old_data};
use crate::notification::exit_with_error_notification;
use crate::{ALERT_SCREEN_ENV_VAR, SCREEN_DATA_CSV_PATH};
use socket2::{Domain, Socket, Type};
use std::error::Error;
use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn create_socket(socket_addr: &String) -> (Socket, TcpListener) {
    let socket = match Socket::new(Domain::IPV6, Type::STREAM, None) {
        Ok(socket) => socket,
        Err(err) => {
            let error_message = format!("Error creating socket: {}", err);
            eprintln!("{}", error_message);
            exit_with_error_notification(error_message.as_str());
        }
    };

    let address: SocketAddr = match socket_addr.parse() {
        Ok(address) => address,
        Err(err) => {
            let error_message = format!("Error parsing socket address: {}", err);
            eprintln!("{}", error_message);
            exit_with_error_notification(error_message.as_str());
        }
    };
    let address = address.into();
    if let Err(err) = socket.bind(&address) {
        let error_message = format!(
            "Error binding socket to address: {}, err: {}",
            &socket_addr, err
        );
        eprintln!("{}", error_message);
        exit_with_error_notification(error_message.as_str());
    }
    if let Err(err) = socket.listen(128) {
        let error_message = format!("Error listening on socket: {}", err);
        eprintln!("{}", error_message);
        exit_with_error_notification(error_message.as_str());
    }
    println!("Listening on {}", &socket_addr);

    let listener = match socket.try_clone() {
        Ok(cloned_socket) => cloned_socket.into(),
        Err(err) => {
            let error_message = format!("Error cloning socket: {}", err);
            eprintln!("{}", error_message);
            exit_with_error_notification(error_message.as_str());
        }
    };

    (socket, listener)
}
pub fn close_socket(socket: Socket) -> io::Result<()> {
    println!("Closing socket...");
    socket.shutdown(Shutdown::Both)
}

pub fn connect_to_socket(socket_addr: String) -> TcpStream {
    let stream = match TcpStream::connect(socket_addr) {
        Ok(stream) => stream,
        Err(err) => {
            let error_message = format!("Error connecting to socket: {}", err);
            eprintln!("{}", error_message);
            exit_with_error_notification(error_message.as_str());
        }
    };
    stream
}
// Send the terminating stream to close socket connection
// When the listen_for_connection loop iterates as there is another stream, it will encounter the changed child_program_finished and break the loop.
pub fn send_terminating_mssg(socket_path: String) {
    let mut stream = connect_to_socket(socket_path);
    match stream.write_all(b"Terminating Stream") {
        Ok(()) => {
            println!("Terminating stream sent.");
        }
        Err(err) => {
            let error_message = format!("Error sending terminating stream: {}", err);
            eprintln!("{}", error_message);
            exit_with_error_notification(error_message.as_str());
        }
    }
    match stream.shutdown(Shutdown::Both) {
        Ok(()) => {
            println!("Stream successfully shutdown.");
        }
        Err(err) => {
            let error_message = format!("Error shutting down stream: {}", err);
            eprintln!("{}", error_message);
            exit_with_error_notification(error_message.as_str());
        }
    }
}
pub fn listen_for_connections(
    listener: &TcpListener,
    terminating_arc: &Arc<AtomicBool>,
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
                handle_client(stream, update_csv, alert_screen_time)?;
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
    mut stream: TcpStream,
    update_csv: &Arc<AtomicBool>,
    alert_screen_time: u64,
) -> Result<(), Box<dyn Error>> {
    let mut received = String::new();
    stream.read_to_string(&mut received)?;
    let update_csv_str = String::from("UPDATE_CSV");
    let path_str = String::from("PATH");
    let health_check_str = String::from("HEALTH_CHECK");
    let alert_screen_env_var_str = ALERT_SCREEN_ENV_VAR.to_string();
    match received {
        s if s == health_check_str => {
            println!("Received HEALTH_CHECK request!");
            stream.write_all(b"Ok")?;
            Ok(())
        }
        s if s == update_csv_str => {
            println!("Received UPDATE_CSV request!");
            update_csv.store(true, Ordering::Relaxed);
            stream.write_all(b"Success")?;
            Ok(())
        }
        s if s == path_str => {
            println!("Received PATH request!");
            let curr_path = get_curr_path_to_csv(&SCREEN_DATA_CSV_PATH.to_string());
            stream.write_all(curr_path.as_bytes())?;
            println!("Sent path! - {}", curr_path);
            Ok(())
        }
        s if s == alert_screen_env_var_str => {
            println!("Received alert screen request!");
            stream.write_all(alert_screen_time.to_string().as_bytes())?;
            Ok(())
        }
        s if (received.len() >= 7) && (&received[..6] == "DELETE") => {
            println!("Received delete request!");
            let months_str = s[7..].trim().to_string();
            let months: u32 = match months_str.parse() {
                Ok(months) => months,
                Err(err) => {
                    eprintln!("Error parsing months: {}", err);
                    stream.write_all(b"Failure")?;
                    return Ok(());
                }
            };
            match remove_old_data(months, &SCREEN_DATA_CSV_PATH.to_string()) {
                Ok(()) => {
                    stream.write_all(b"Success")?;
                    println!("Successfully removed old data!");
                }
                Err(err) => {
                    eprintln!("Error removing old data: {}", err);
                    stream.write_all(b"Failure")?;
                }
            }
            Ok(())
        }
        _ => {
            println!("Received unknown request: {}", received);
            Ok(())
        }
    }
}
