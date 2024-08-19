//! This module manages the TCP server and how/where the packets are managed/sent.

use crate::config;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

// TODO: Logging.

/// Global buffer/packet size allocation when allocating for a new packet buffer (in bytes).
const BUFFER_SIZE: usize = 1024;

/// This function listens for every incomming TCP connection/packet.
#[tokio::main]
pub async fn listen() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Settings::new();
    let server_address = format!("0.0.0.0:{}", config.server_port);
    let listener = TcpListener::bind(server_address).await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        handle_connection(socket, addr).await;
    }
}

/// This function handles each connection.
async fn handle_connection(mut socket: TcpStream, addr: SocketAddr) {
    tokio::spawn(async move {
        println!("New connection: {}", addr);

        // TODO: Maybe have a bigger/dynamic buffer.
        let mut buf = [0; BUFFER_SIZE];

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(n) if n == 0 => return,
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Failed to read from socket: {}", e);
                    return;
                }
            };

            if let Err(e) = socket.write_all(&buf[0..n]).await {
                eprintln!("Failed to write to socket: {}", e);
                return;
            }

            // handle the packet!
            handle_packet(buf, n).await;
        }
    });
}

/// This function takes the buffer, an array of bytes,
async fn handle_packet(buffer: [u8; BUFFER_SIZE], length: usize) {
    println!("Reveived packet of length: {length}: ");

    for (idx, byte) in buffer.iter().enumerate().take(length) {
        println!("{idx}: {byte}");
    }
}
