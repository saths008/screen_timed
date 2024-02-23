use active_win_pos_rs::get_active_window;
use csv::ReaderBuilder;
use csv::WriterBuilder;
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fmt, thread, time};
#[derive(Debug, Clone)]
struct CSVWriterError;
impl fmt::Display for CSVWriterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing CSV or CSV Writer error")
    }
}
#[derive(Debug, Clone)]
struct CSVReaderError;
impl fmt::Display for CSVReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing CSV or CSV Reader error")
    }
}

fn read_csv(csv_path: String) -> Result<bool, CSVReaderError> {
    let rdr_result = ReaderBuilder::new().from_path(csv_path);
    let mut rdr = match rdr_result {
        Ok(rdr_arm) => rdr_arm,
        Err(_) => {
            return Err(CSVReaderError);
        }
    };
    for result in rdr.records() {
        let record_res = result;
        let record = match record_res {
            Ok(record_arm) => record_arm,
            Err(_) => {
                return Err(CSVReaderError);
            }
        };

        println!("{:?}", record);
    }
    Ok(true)
}
fn write_data_to_csv(
    program_times: &HashMap<String, time::Duration>,
    csv_path: String,
) -> Result<bool, CSVWriterError> {
    if !Path::new(&csv_path).exists() {
        return Err(CSVWriterError);
    }
    let mut wtr_result = WriterBuilder::new().from_path("foo.csv");
    let mut wtr = match wtr_result {
        Ok(wtr_arm) => wtr_arm,
        Err(_) => {
            return Err(CSVWriterError);
        }
    };

    Ok(true)
}
fn screen_time_daemon(program_times: &mut HashMap<String, time::Duration>) {
    match get_active_window() {
        Ok(active_window) => {
            let app_name = &active_window.app_name;
            if program_times.contains_key(app_name) {
                let existing_time = program_times.get(app_name).unwrap();
                let new_time = *existing_time + time::Duration::from_secs(1);
                program_times.insert(app_name.to_string(), new_time);
            } else {
                program_times.insert(app_name.to_string(), time::Duration::from_secs(1));
            }
            // println!("app name: {}", app_name);
            // println!("active window: {:#?}", active_window);
        }
        Err(()) => {
            println!("error occurred while getting the active window");
        }
    }
}

fn main() -> Result<(), io::Error> {
    let mut program_times: HashMap<String, time::Duration> = HashMap::new();
    let term = Arc::new(AtomicBool::new(false));
    // signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;
    //
    // Keep executing as long the SIGTERM has not been called.
    while !term.load(Ordering::Relaxed) {
        thread::sleep(time::Duration::from_secs(1));
        screen_time_daemon(&mut program_times);
    }

    println!("Sigint received!");
    for (program_name, duration) in &program_times {
        println!("{}: {}", program_name, duration.as_secs());
    }
    Ok(())
}
