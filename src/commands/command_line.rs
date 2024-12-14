use std::{thread, time::Duration};

use colored::Colorize;
use log::{debug, info, warn};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::{consts, fs_manager, gracefully_exit, player};

// Asynchronously handles user input. It never returns

// TODO: IMPLEMENT COMMANDS SEPARATELY FROM THIS FUNCTION, otherwise the code will just be as good as a dumpster fire
// TODO: use the 'Command Pattern' and command handlers
pub async fn handle_input() -> ! {
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut buffer = String::new();

    loop {
        buffer.clear();
        if let Ok(bytes_read) = reader.read_line(&mut buffer).await {
            if bytes_read == 0 {
                continue; // EOF
            }
        }

        debug!("you entered: {buffer}");
        // Debug/test logic down here

        if buffer.trim().to_lowercase() == "stop" {
            let content = "Server will stop in few second…";
            warn!("{}", content.red().bold());
            gracefully_exit(crate::ExitCode::Failure)
        }
        //made a server operator (level 4)

        if buffer.trim().to_lowercase().starts_with("op") {
            let mut parts = buffer.split_whitespace();
            parts.next();

            if let Some(element) = parts.next() {
                let uuid = match player::get_uuid(element).await {
                    Ok(body) => body,
                    Err(_) => String::from("not found"),
                };
                let content = match fs_manager::write_ops_json(
                    consts::file_paths::OPERATORS,
                    uuid.as_str(),
                    element,
                    4,
                    true,
                ) {
                    Ok(_) => format!("Made {} a server operator.", element),
                    Err(e) => format!(
                        "Failed to make {} as a server operator, error: {} ",
                        element, e
                    ),
                };
                info!("{}", content);
            } else {
                warn!("Missing one argument: op <-")
            }
        }
    }
}
