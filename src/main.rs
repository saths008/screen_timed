use active_win_pos_rs::get_active_window;
use csv::ReaderBuilder;
// use csv::WriterBuilder;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

fn read_csv(csv_path: String) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().from_path(csv_path)?;
    for result in rdr.records() {
        let record = result?;
        println!("{:?}", record);
    }
    println!("Finish read_csv method");
    Ok(())
}
// fn write_data_to_csv(
//     program_times: &HashMap<String, time::Duration>,
//     csv_path: String,
// ) -> Result<(), Box<dyn Error>> {
//     let mut wtr_result = WriterBuilder::new().from_path(csv_path)?;
//     Ok(())
// }
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
    let read_result = read_csv("screen_time_data.csv".to_string());
    match read_result {
        Ok(_) => println!("read csv success"),
        Err(e) => println!("error reading csv: {}", e),
    }
    Ok(())
}
