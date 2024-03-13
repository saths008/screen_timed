#[cfg(test)]
pub mod tests {
    use std::{env, fs::File, io::Write};

    use tempfile;
    pub const CSV_NAME: &str = "screen_time_data.csv";
    pub const SOCKET_NAME: &str = "screen-time-sock";

    pub fn get_socket_path(temp_dir: &tempfile::TempDir) -> String {
        String::from(
            temp_dir
                .path()
                .join(&SOCKET_NAME.to_string())
                .to_str()
                .unwrap(),
        )
    }

    fn create_and_set_temp_dir() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        env::set_current_dir(&temp_dir).expect("Failed to set current directory");
        temp_dir
    }
    fn create_env_file(temp_dir: &tempfile::TempDir) {
        let env_file_path = temp_dir.path().join(".env");
        let mut file = File::create(&env_file_path).expect("Failed to create .env file");

        file.write_all(b"ALERT_SCREEN=45\n")
            .expect("Failed to write to .env file");
    }
    pub fn setup() -> (tempfile::TempDir, String) {
        let temp_dir = create_and_set_temp_dir();
        let csv_name = CSV_NAME.to_string();
        let actual_path_to_csv = String::from(temp_dir.path().join(&csv_name).to_str().unwrap());
        println!("actual_path_to_csv: {}", actual_path_to_csv);
        create_env_file(&temp_dir);

        (temp_dir, actual_path_to_csv)
    }
}
