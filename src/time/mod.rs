use chrono::{DateTime, Local, Utc};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

/// Returns a formatted time string.
pub fn get_formatted_time() -> String {
    let now = Utc::now();
    let local_time: DateTime<Local> = now.with_timezone(&Local); // Convert to local machine time
    local_time.format("%a %b %d %H:%M:%S %Y").to_string() // Format time
}
//returns a none formatted time.
pub fn get_time() -> DateTime<Local> {
    let now = Utc::now();
    now.with_timezone(&Local) // Convert to local machine time
}

/// Returns the current time formatted using the ISO-8601 standard.
pub fn get_time_iso() -> Result<String, Box<dyn std::error::Error>> {
    // Local time in ISO-8601 with numeric offset, e.g. 2025-08-21T14:34:56+02:00
    let local = OffsetDateTime::now_local()?;
    Ok(local.format(&Iso8601::DEFAULT)?)
}
