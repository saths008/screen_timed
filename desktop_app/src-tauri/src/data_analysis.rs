use csv::ReaderBuilder;
use serde_derive::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::TimestampSeconds;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{error::Error, vec};

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Row {
    #[serde_as(as = "TimestampSeconds<i64>")]
    timestamp: SystemTime,
    application: String,
    //How long in seconds the application was active
    duration: u64,
}
#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
pub struct RowDetails {
    #[serde_as(as = "TimestampSeconds<i64>")]
    timestamp: SystemTime,
    //How long in seconds the application was active
    duration: u64,
}
pub fn week_screen_time(
    csv_path: String,
    start_of_week: u64,
) -> Result<Vec<Vec<Row>>, Box<dyn Error>> {
    let mut week_rows: Vec<Vec<Row>> = vec![Vec::new(); 7];
    for day in 0..7 {
        let day_start = start_of_week + (day * 24 * 60 * 60);
        println!("Day start: {}", day_start);
        let day_rows = date_screen_time(csv_path.clone(), day_start)?;
        week_rows[day as usize] = day_rows;
    }
    Ok(week_rows)
}
pub fn date_screen_time(
    csv_path: String,
    start_of_date: u64,
) -> Result<vec::Vec<Row>, Box<dyn Error>> {
    println!("csv_path: {}", csv_path);
    let mut rdr = ReaderBuilder::new().from_path(csv_path)?;

    let end_of_date = start_of_date + (24 * 60 * 60);
    println!("Start of date: {}", start_of_date);
    println!("End of date: {}", end_of_date);

    let mut records_map: HashMap<String, RowDetails> = HashMap::new();
    let mut records: Vec<Row> = Vec::new(); // Collect records into a vector
    for result in rdr.deserialize() {
        let record: Row = result?;
        records.push(record); // Collect each record into the vector
    }
    for record in records.iter().rev() {
        let record_timestamp = record.timestamp.duration_since(UNIX_EPOCH)?;
        let is_date = record_timestamp >= std::time::Duration::from_secs(start_of_date)
            && record_timestamp <= std::time::Duration::from_secs(end_of_date);
        if is_date {
            // println!("{:?}", &record);
            let app_name = &record.application;
            if records_map.contains_key(app_name) {
                let row_details = records_map.get_mut(app_name).unwrap();
                (row_details).duration += record.duration;
            } else {
                records_map.insert(
                    app_name.to_string(),
                    RowDetails {
                        timestamp: record.timestamp,
                        duration: record.duration,
                    },
                );
            }
        } else {
            // println!("Not today");
        }
    }

    println!("Finish read_csv method");
    Ok(records_map
        .iter()
        .map(|(app_name, row_details)| Row {
            timestamp: row_details.timestamp,
            application: app_name.to_string(),
            duration: row_details.duration,
        })
        .collect())
}
