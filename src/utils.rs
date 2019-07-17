use std::time::Duration;

pub fn format_time(instant: Duration) -> String {
    let seconds = instant.as_secs();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
