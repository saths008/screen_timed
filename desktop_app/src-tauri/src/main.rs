// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod data_analysis;
mod socket_comm;
use data_analysis::Row;
use std::vec;

fn get_csv_path() -> Result<String, String> {
    let csv_path = match socket_comm::get_path_message() {
        Ok(path) => path,
        Err(e) => {
            println!("Error while getting path from socket: {}", e);
            return Err("Error while getting path from socket".to_string());
        }
    };
    Ok(csv_path)
}

#[tauri::command(rename_all = "snake_case")]
fn get_week_screen_time(start_of_date: u64) -> Result<vec::Vec<vec::Vec<Row>>, String> {
    let csv_path = get_csv_path()?;

    match data_analysis::week_screen_time(csv_path, start_of_date) {
        Ok(records) => Ok(records),
        Err(e) => {
            println!("Error while reading csv: {}", e);
            Err("Error while reading csv".to_string())
        }
    }
}
#[tauri::command(rename_all = "snake_case")]
fn send_get_health_check_message() -> Result<String, String> {
    match socket_comm::get_health_check_message() {
        Ok(health_check_message) => Ok(health_check_message),
        Err(e) => {
            println!("Error while sending message to socket: {}", e);
            Err("Error getting health check message".to_string())
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
fn send_get_alert_screen_time_message() -> Result<u64, String> {
    match socket_comm::get_alert_screen_time_message() {
        Ok(alert_screen_time) => Ok(alert_screen_time),
        Err(e) => {
            println!("Error while sending message to socket: {}", e);
            Err("Error while sending update message to socket".to_string())
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
fn send_update_socket_message() -> Result<(), String> {
    match socket_comm::send_update_message() {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error while sending message to socket: {}", e);
            Err("Error while sending update message to socket".to_string())
        }
    }
}
#[tauri::command(rename_all = "snake_case")]
fn send_delete_months_data_message(months: u32) -> Result<(), String> {
    match socket_comm::delete_months_data_message(months) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error while sending message to socket: {}", e);
            Err("Error while sending update message to socket".to_string())
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
fn get_date_screen_time(start_of_date: u64) -> Result<vec::Vec<Row>, String> {
    let csv_path = get_csv_path()?;
    println!("get_date_screen_time fn called with: {}", start_of_date);
    match data_analysis::date_screen_time(csv_path, start_of_date) {
        Ok(records) => Ok(records),
        Err(e) => {
            println!("Error while reading csv: {}", e);
            Err("Error while reading csv".to_string())
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_date_screen_time,
            get_week_screen_time,
            send_update_socket_message,
            send_get_alert_screen_time_message,
            send_delete_months_data_message,
            send_get_health_check_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
