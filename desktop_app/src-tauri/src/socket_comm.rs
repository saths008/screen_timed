use std::error::Error;
use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};

const SOCKET_ADDR: &str = "[::1]:12345";
const ALERT_SCREEN_ENV_VAR: &str = "ALERT_SCREEN";

fn send_message_to_socket(message: String) -> Result<String, Box<dyn Error>> {
    let mut stream = TcpStream::connect(SOCKET_ADDR.to_string())?;
    stream.write_all(message.as_bytes())?;
    stream.shutdown(Shutdown::Write)?;
    let mut received = String::new();
    stream.read_to_string(&mut received)?;
    Ok(received)
}
pub fn get_health_check_message() -> Result<String, Box<dyn Error>> {
    let received = send_message_to_socket("HEALTH_CHECK".to_string())?;
    Ok(received)
}
pub fn get_path_message() -> Result<String, Box<dyn Error>> {
    let received = send_message_to_socket("PATH".to_string())?;
    Ok(received)
}

pub fn send_update_message() -> Result<(), Box<dyn Error>> {
    let received = send_message_to_socket("UPDATE_CSV".to_string())?;
    if received.trim() == "Success" {
        Ok(())
    } else {
        Err("Failed to update csv".into())
    }
}
pub fn get_alert_screen_time_message() -> Result<u64, Box<dyn Error>> {
    let mut alert_screen_str = send_message_to_socket(ALERT_SCREEN_ENV_VAR.to_string())?;

    alert_screen_str = alert_screen_str.trim().to_string();
    let alert_screen_time: u64 = alert_screen_str.parse()?;
    Ok(alert_screen_time)
}
pub fn delete_months_data_message(months: u32) -> Result<(), Box<dyn Error>> {
    let message = format!("DELETE {}", months);
    let response = send_message_to_socket(message)?;

    if response.trim() == "Failure" {
        Err("Failed to delete data".into())
    } else if response.trim() == "Success" {
        Ok(())
    } else {
        Err("Unknown response".into())
    }
}
