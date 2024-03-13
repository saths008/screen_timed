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
use crate::SCREEN_DATA_CSV_PATH;

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Row {
    #[serde_as(as = "TimestampSeconds<i64>")]
    timestamp: SystemTime,
    application: String,
    //How long in seconds the application was active
    duration: u64,
}
pub fn get_curr_path_to_csv() -> String {
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

pub fn write_data_to_csv(
    program_times: &HashMap<String, time::Duration>,
    csv_path: &String,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new().append(true).open(csv_path)?;
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

pub fn remove_old_data(months: u32, csv_path: &String) -> Result<(), Box<dyn Error>> {
    let backup_screen_csv_path = format!("backup_{}", csv_path);
    copy(SCREEN_DATA_CSV_PATH, &backup_screen_csv_path)?;
    let new_screen_csv_path = format!("new_{}", csv_path);

    File::create(&new_screen_csv_path)?;
    let mut rdr = ReaderBuilder::new().from_path(&backup_screen_csv_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_path(&new_screen_csv_path)?;

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
    rename(new_screen_csv_path, SCREEN_DATA_CSV_PATH)?;
    remove_file(backup_screen_csv_path)?;
    println!("Successfully removed {} months old data", months);
    Ok(())
}
