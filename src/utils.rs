use std::time::Duration;

pub fn format_time(instant: Duration) -> String {
    let seconds = instant.as_secs();
    let hours = seconds / 3600;
    let minutes = (seconds - (hours * 3600)) / 60;

    let hours_fmt = if hours < 10 { format!("0{}", hours.to_string()) } else { hours.to_string() };
    let minutes_fmt = if minutes < 10 { format!("0{}", minutes.to_string()) } else { minutes.to_string() };
    let seconds_fmt = if seconds < 10 { format!("0{}", seconds.to_string()) } else { seconds.to_string() };

    format!("{}:{}:{}", hours_fmt, minutes_fmt, seconds_fmt)
}
