use log::info;

use crate::shutdown::{self, ExitCode};

/// Sets up a behavior when the user executes CTRL + C.
pub fn init_ctrlc_handler() -> Result<(), Box<dyn std::error::Error>> {
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down...");
        shutdown::gracefully_exit(ExitCode::CtrlC);
    })?;

    Ok(())
}
