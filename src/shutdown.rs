use log::{info, warn};

use crate::consts::messages;

/// Enum representing standardized server exit codes.
pub enum ExitCode {
    Success,
    Failure,
    CtrlC,
}

/// Gracefully exits the server with an exit code.
pub fn gracefully_exit(exit_code: ExitCode) -> ! {
    let numerical_exit_code: i32 = match exit_code {
        ExitCode::Success => {
            info!("{}", *messages::SERVER_SHUTDOWN_SUCCESS);
            0
        }
        ExitCode::Failure => {
            warn!("{}", *messages::SERVER_SHUTDOWN_ERROR);
            1
        }
        ExitCode::CtrlC => {
            info!("{}", *messages::SERVER_SHUTDOWN_CTRL_C);
            130
        }
    };

    std::process::exit(numerical_exit_code);
}
