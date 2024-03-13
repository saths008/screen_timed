use std::{env, path::PathBuf, string::String};

use crate::SCREEN_DATA_CSV_PATH;
use crate::{notification::exit_with_error_notification, ALERT_SCREEN_ENV_VAR};

#[derive(Debug)]
pub struct Config {
    alert_screen_time: u64,
}

impl Config {
    pub fn build(alert_screen_time: &str) -> Result<Config, &'static str> {
        let alert_screen_time: u64 = match alert_screen_time.trim().to_string().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Please enter a valid number");
                return Err("Invalid work session duration");
            }
        };

        Ok(Config { alert_screen_time })
    }

    pub fn get_alert_screen_time(&self) -> u64 {
        self.alert_screen_time
    }

    pub fn print_out_config(&self) {
        println!("Alert Screen Time: {}.", self.get_alert_screen_time());
    }
}

pub fn get_curr_path() -> String {
    let current_path: PathBuf = match env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            exit_with_error_notification(
                format!("Exiting: Error getting current path: {}", err).as_str(),
            );
        }
    };
    let current_path_str = match current_path.to_str() {
        Some(path) => {
            let mut full_path = path.to_string();
            full_path.push_str(format!("/{}", SCREEN_DATA_CSV_PATH.to_string()).as_str());
            full_path
        }
        None => {
            exit_with_error_notification("Error getting the current path!");
        }
    };
    current_path_str
}
pub fn new_config() -> Config {
    if let Err(err) = dotenvy::dotenv() {
        exit_with_error_notification(format!("Error loading .env file: {}", err).as_str());
    }

    let alert_screen_env_str: String = match dotenvy::var(ALERT_SCREEN_ENV_VAR) {
        Ok(alert_screen_env) => alert_screen_env,
        Err(err) => {
            exit_with_error_notification(
                format!("Error getting ALERT_SCREEN_ENV_VAR: {}", err).as_str(),
            );
        }
    };

    let config = match Config::build(&alert_screen_env_str) {
        Ok(config) => config,
        Err(err) => {
            exit_with_error_notification(
                format!("Error parsing ALERT_SCREEN_ENV_VAR: {}", err).as_str(),
            );
        }
    };

    config.print_out_config();
    config
}
