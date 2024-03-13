use std::process::exit;

use notify_rust::Notification;
use notify_rust::Timeout;

pub fn screen_time_notification(alert_screen_time: u64) {
    let alert_message = format!(
        "You have been on the screen for {} minutes",
        alert_screen_time
    );
    if let Err(err) = Notification::new()
        .summary("Screen Time Alert")
        .body(alert_message.as_str())
        .timeout(Timeout::Never)
        .show()
    {
        eprintln!("Error showing notification: {}", err);
        exit(1);
    }
}

// Send error notification to user, then process exit with code 1.
pub fn exit_with_error_notification(error_message: &str) -> ! {
    eprintln!("{}", &error_message);
    if let Err(err) = Notification::new()
        .summary("Error")
        .body(error_message)
        .timeout(Timeout::Never)
        .show()
    {
        eprintln!("Error showing notification: {}", err);
    }

    exit(1);
}
