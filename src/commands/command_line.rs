use log::debug;
use tokio::io::{self, AsyncBufReadExt, BufReader};

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
    }
}
