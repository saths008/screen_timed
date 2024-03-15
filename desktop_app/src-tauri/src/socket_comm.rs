use std::error::Error;
use std::io::prelude::*;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/screen-time-sock";
const ALERT_SCREEN_ENV_VAR: &str = "ALERT_SCREEN";

pub fn get_path_message() -> Result<String, Box<dyn Error>> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.write_all(b"PATH")?;
    stream.shutdown(Shutdown::Write)?;
    let mut received = String::new();
    stream.read_to_string(&mut received)?;
    println!("{}", received);
    stream.shutdown(Shutdown::Read)?;

    Ok(received)
}

pub fn send_update_message() -> Result<(), Box<dyn Error>> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.write_all(b"UPDATE_CSV")?;
    stream.shutdown(Shutdown::Write)?;
    let mut received = String::new();
    stream.read_to_string(&mut received)?;
    println!("{}", received);
    stream.shutdown(Shutdown::Read)?;

    Ok(())
}
pub fn get_alert_screen_time_message() -> Result<u64, Box<dyn Error>> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.write_all(ALERT_SCREEN_ENV_VAR.as_bytes())?;
    stream.shutdown(Shutdown::Write)?;
    let mut alert_screen_str = String::new();
    stream.read_to_string(&mut alert_screen_str)?;
    println!("{}", alert_screen_str);
    stream.shutdown(Shutdown::Read)?;

    alert_screen_str = alert_screen_str.trim().to_string();
    let alert_screen_time: u64 = alert_screen_str.parse()?;
    Ok(alert_screen_time)
}
pub fn delete_months_data_message(months: u32) -> Result<(), Box<dyn Error>> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    let message = format!("DELETE {}", months);
    stream.write_all(message.as_bytes())?;
    stream.shutdown(Shutdown::Write)?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("{}", response);
    stream.shutdown(Shutdown::Read)?;

    if response.trim() == "Failure" {
        Err("Failed to delete data".into())
    } else if response.trim() == "Success" {
        Ok(())
    } else {
        Err("Unknown response".into())
    }
}
