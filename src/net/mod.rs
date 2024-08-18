//! This module manages the TCP server and how/where the packets are managed/sent.

use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
pub async fn listen() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("");

    Ok(())
}
