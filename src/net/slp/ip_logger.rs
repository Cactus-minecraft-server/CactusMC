//! Manages the logging of IPs passed down from the SLP handler.

use log::error;
use std::path::Path;

/// Holds Notchian client information that will be logged into the logged_ips file.
pub struct ClientIpLoggerInfo {
    pub ip: String,
}

/// Makes the line that will be appended to the logged_ips file.
fn make_line(info: &ClientIpLoggerInfo) -> String {
    let time: String = crate::time::get_time_iso().unwrap();
    format!("{},{}\n", time, info.ip)
}

/// Logs an IP to the logged_ips file. This function should be called in the SLP after
/// receiving positive confirmation the client is a Notchian Minecraft server.
/// This function is low-friction, does not propagate errors.
pub fn log_ip(info: ClientIpLoggerInfo) {
    let path: &Path = Path::new(crate::consts::file_paths::LOGGED_IPS);
    if !path.is_file() {
        if let Err(e) = crate::fs_manager::utils::create_file(path, "") {
            error!("Failed to log IP: failed to create {path:#?}: {e}");
            return;
        }
    }

    // Check a second time to make sure the function really created the file.
    if !path.is_file() {
        error!("Failed to log IP: {path:#?} does not exist");
        return;
    }

    // append file with IP, date,
    if let Err(e) = crate::fs_manager::utils::append_file(path, &make_line(&info)) {
        error!("Failed to log IP: {e}");
    }
}
