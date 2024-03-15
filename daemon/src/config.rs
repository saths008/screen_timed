use std::string::String;

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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    fn generate_random_number() -> u64 {
        rand::thread_rng().gen()
    }
    fn generate_random_number_as_str() -> (u64, String) {
        let rand_number = generate_random_number();
        let rand_number_as_str = rand_number.to_string();
        (rand_number, rand_number_as_str)
    }
    #[test]
    fn build_config_passes_with_valid_inputs() {
        let (rand_number, rand_number_as_str) = generate_random_number_as_str();
        let config = Config::build(rand_number_as_str.as_str()).unwrap();
        assert_eq!(config.get_alert_screen_time(), rand_number);
    }
    #[test]
    fn get_alert_screen_time_is_correct() {
        let (_, rand_number_as_str) = generate_random_number_as_str();
        let config = Config::build(rand_number_as_str.as_str()).unwrap();
        assert_eq!(config.get_alert_screen_time(), config.alert_screen_time);
    }

    #[test]
    fn env_file_is_read_correctly() {
        let config = new_config();
        assert_eq!(config.alert_screen_time, 45);
    }
}
