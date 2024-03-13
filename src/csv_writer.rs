use csv::ReaderBuilder;
use csv::WriterBuilder;
use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::TimestampSeconds;
use std::env;
use std::fs::{copy, remove_file, rename, File, OpenOptions};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use std::{
    collections::HashMap,
    error::Error,
    time::{self, SystemTime},
};

use crate::notification::exit_with_error_notification;

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Row {
    #[serde_as(as = "TimestampSeconds<i64>")]
    timestamp: SystemTime,
    application: String,
    //How long in seconds the application was active
    duration: u64,
}
pub fn get_curr_path_to_csv(csv_path: &String) -> String {
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
            full_path.push_str(format!("/{}", csv_path).as_str());
            full_path
        }
        None => {
            exit_with_error_notification("Error getting the current path!");
        }
    };
    current_path_str
}
pub fn write_data_to_csv(
    program_times: &HashMap<String, time::Duration>,
    csv_name: &String,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new().append(true).open(csv_name)?;
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);
    for (program_name, duration) in program_times {
        wtr.serialize(Row {
            timestamp: SystemTime::now(),
            application: program_name.to_string(),
            duration: duration.as_secs(),
        })?;
    }
    wtr.flush()?;
    Ok(())
}

//Removes one month of the oldest data
pub fn remove_old_data(months: u32, csv_name: &String) -> Result<(), Box<dyn Error>> {
    let backup_screen_csv_name = format!("backup_{}", csv_name);
    copy(csv_name, &backup_screen_csv_name)?;
    let new_screen_csv_name = format!("new_{}", csv_name);

    File::create(&new_screen_csv_name)?;
    let mut rdr = ReaderBuilder::new().from_path(&backup_screen_csv_name)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_path(&new_screen_csv_name)?;

    let mut rdr_iter = rdr.deserialize();
    let first_result_iter = rdr_iter.next();
    if first_result_iter.is_none() {
        return Ok(());
    }
    let first_result: Row = first_result_iter.unwrap()?;
    let first_timestamp = first_result.timestamp.duration_since(UNIX_EPOCH)?;
    // 30 days approximation in a month
    let end_timestamp =
        first_timestamp + time::Duration::from_secs((60 * 60 * 24 * 30 * months).into());
    for result in rdr_iter {
        let record: Row = result?;
        let timestamp = record.timestamp.duration_since(UNIX_EPOCH)?;
        let should_delete = timestamp >= first_timestamp && timestamp <= end_timestamp;
        if !should_delete {
            wtr.serialize(record)?;
        }
    }
    wtr.flush()?;
    //replace old csv with new csv
    rename(new_screen_csv_name, csv_name)?;
    remove_file(backup_screen_csv_name)?;
    println!("Successfully removed {} months old data", months);
    Ok(())
}
#[cfg(test)]
mod tests {
    use std::{io::Write, time::Duration};

    use super::*;
    use serial_test::serial;
    use tempfile;
    const CSV_NAME: &str = "screen_time_data.csv";

    //helper to read csv
    fn read_csv(csv_path: &String) -> Result<Vec<Row>, Box<dyn Error>> {
        let mut rdr = ReaderBuilder::new().from_path(csv_path)?;
        let mut records: Vec<Row> = Vec::new();
        for result in rdr.deserialize() {
            let record: Row = result?;
            records.push(record);
        }
        Ok(records)
    }

    fn write_header_to_csv(csv_path: &String, timestamp: SystemTime) -> Result<(), Box<dyn Error>> {
        let file = OpenOptions::new().write(true).create(true).open(csv_path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        wtr.serialize(Row {
            timestamp,
            application: "Application".to_string(),
            duration: 0,
        })?;
        wtr.flush()?;
        Ok(())
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
    fn setup() -> (tempfile::TempDir, String) {
        let temp_dir = create_and_set_temp_dir();
        let csv_name = CSV_NAME.to_string();
        let actual_path_to_csv = String::from(temp_dir.path().join(&csv_name).to_str().unwrap());
        println!("actual_path_to_csv: {}", actual_path_to_csv);
        create_env_file(&temp_dir);

        (temp_dir, actual_path_to_csv)
    }
    #[test]
    #[serial]
    fn test_get_curr_path_to_csv() {
        //temp_dir is dropped when out of scope and deletes the temp dir
        let (_temp_dir, actual_path_to_csv) = setup();
        let csv_name = CSV_NAME.to_string();
        println!("In test_get_curr_path_to_csv");
        let path_to_csv = get_curr_path_to_csv(&csv_name);
        println!("path_to_csv: {}", path_to_csv);
        println!("expected_path: {}", actual_path_to_csv);
        assert_eq!(path_to_csv, actual_path_to_csv);
    }

    #[test]
    #[serial]
    fn test_write_headers_to_csv() {
        let (_temp_dir, actual_path_to_csv) = setup();
        write_header_to_csv(&actual_path_to_csv, SystemTime::now()).unwrap();
        let rows_vector = read_csv(&actual_path_to_csv).unwrap();
        println!("rows_vector[0]: {:?}", rows_vector[0]);
        assert_eq!(rows_vector.len(), 1);
        assert_eq!(rows_vector[0].application, "Application");
        assert_eq!(rows_vector[0].duration, 0);
    }

    #[test]
    #[serial]
    fn test_write_data_to_csv() {
        let (_temp_dir, actual_path_to_csv) = setup();
        write_header_to_csv(&actual_path_to_csv, SystemTime::now()).unwrap();
        let mut program_times: HashMap<String, time::Duration> = HashMap::new();
        program_times.insert("Test".to_string(), time::Duration::from_secs(10));
        write_data_to_csv(&program_times, &CSV_NAME.to_string()).unwrap();
        let rows_vector = read_csv(&actual_path_to_csv).unwrap();

        println!("rows_vector: {:?}", rows_vector);
        assert_eq!(rows_vector.len(), 2);
        assert_eq!(rows_vector[0].application, "Application");
        assert_eq!(rows_vector[0].duration, 0);
        assert_eq!(rows_vector[1].application, "Test");
        assert_eq!(rows_vector[1].duration, 10);
    }

    #[test]
    #[serial]
    fn test_remove_all_data() {
        let (_temp_dir, actual_path_to_csv) = setup();
        write_header_to_csv(&actual_path_to_csv, SystemTime::now()).unwrap();
        let mut program_times: HashMap<String, time::Duration> = HashMap::new();
        program_times.insert("Test".to_string(), time::Duration::from_secs(10));
        write_data_to_csv(&program_times, &CSV_NAME.to_string()).unwrap();
        let mut rows_vector = read_csv(&actual_path_to_csv).unwrap();
        assert_eq!(rows_vector.len(), 2);

        //Remove the oldest month of data, which deletes everything
        remove_old_data(1, &CSV_NAME.to_string()).unwrap();
        rows_vector = read_csv(&actual_path_to_csv).unwrap();

        println!("rows_vector after removal: {:?}", rows_vector);
        assert_eq!(rows_vector.len(), 0);
    }
    #[test]
    #[serial]
    fn test_remove_one_month_data() {
        let (_temp_dir, actual_path_to_csv) = setup();
        let now = SystemTime::now();
        let one_month = Duration::from_secs(60 * 60 * 24 * 30);
        let two_months_ago = now.checked_sub(2 * one_month).unwrap();
        write_header_to_csv(&actual_path_to_csv, two_months_ago).unwrap();

        let mut program_times: HashMap<String, time::Duration> = HashMap::new();
        program_times.insert("Test".to_string(), time::Duration::from_secs(10));
        write_data_to_csv(&program_times, &CSV_NAME.to_string()).unwrap();

        let mut rows_vector = read_csv(&actual_path_to_csv).unwrap();
        assert_eq!(rows_vector.len(), 2);

        remove_old_data(1, &CSV_NAME.to_string()).unwrap();
        rows_vector = read_csv(&actual_path_to_csv).unwrap();

        println!("rows_vector after removal: {:?}", rows_vector);
        assert_eq!(rows_vector.len(), 1);
        assert_eq!(rows_vector[0].application, "Test");
        assert_eq!(rows_vector[0].duration, 10);
    }
}
