use std::{collections::HashMap, time::Duration};

use active_win_pos_rs::get_active_window;

use crate::notification::exit_with_error_notification;

//Get the current active window and update the current app's time.
pub fn update_current_app(program_times: &mut HashMap<String, Duration>) {
    match get_active_window() {
        Ok(active_window) => {
            let app_name = &active_window.app_name;
            if program_times.contains_key(app_name) {
                let existing_time = match program_times.get(app_name) {
                    Some(time) => time,
                    None => {
                        exit_with_error_notification("Error getting existing time");
                    }
                };
                let new_time = *existing_time + Duration::from_secs(1);
                program_times.insert(app_name.to_string(), new_time);
            } else {
                program_times.insert(app_name.to_string(), Duration::from_secs(1));
            }
        }
        Err(()) => {
            //Could happen when switching windows.
            println!("error occurred while getting the active window");
        }
    }
}
