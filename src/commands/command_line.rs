use std::{thread, time::Duration};

use colored::Colorize;
use log::{debug, info, warn};
use tokio::io::{AsyncBufReadExt, BufReader};

// Asynchronously handles user input. It never returns
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
            warn!("{}",content.red().bold());
            thread::sleep(Duration::from_secs(1));
            crate::gracefully_exit(-1000);
        }
    }
}
