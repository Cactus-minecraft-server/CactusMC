use colored::Colorize;
use log::{debug, info, warn};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::{config::Settings, consts, fs_manager, gracefully_exit, player};

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

        if buffer.trim().to_lowercase() == "stop" {
            let content = "Server will stop in few second…";
            warn!("{}", content.red().bold());
            gracefully_exit(crate::ExitCode::Success)
        }

        if buffer.trim().to_lowercase().starts_with("op") {
            let mut parts = buffer.split_whitespace();
            parts.next(); // Ignore the "op" command itself

            if let Some(player_name) = parts.next() {
                match player::get_uuid(player_name).await {
                    Ok(uuid) if !uuid.is_empty() => {
                        // If player exist try to put it into ops.json.
                        let content = match fs_manager::write_ops_json(
                            consts::file_paths::OPERATORS,
                            &uuid,
                            player_name,
                            Settings::new().op_permission_level,
                            true,
                        ) {
                            Ok(_) => "".to_string(),
                            Err(e) => format!(
                                "Failed to make {} a server operator, error: {}",
                                player_name, e
                            ),
                        };
                        info!("{}", content);
                    }
                    _ => {
                        // Invalid player
                        warn!("That player does not exist");
                    }
                }
            } else {
                warn!("Missing one argument: op <player_name>");
            }
        }
    }
}
