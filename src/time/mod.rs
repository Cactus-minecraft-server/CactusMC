use chrono::{DateTime, Local, Utc};

/// Returns a formatted time string.
pub fn get_formatted_time() -> String {
    let now = Utc::now();
    let local_time: DateTime<Local> = now.with_timezone(&Local); // Convert to local machine time
    local_time.format("%a %b %d %H:%M:%S %Y").to_string() // Format time
}
pub fn get_time() -> DateTime<Local> {
    let now = Utc::now();
    now.with_timezone(&Local) // Convert to local machine time
}
