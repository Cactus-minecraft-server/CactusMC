//! This module manages the TCP server and how/where the packets are managed/sent.

use crate::config;
use crate::packet::utils::print;
use crate::packet::{utils, Packet};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

// All data sent over the network (except for VarInt and VarLong)

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
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
}

enum ConnectionState {
    Handshake,
}

struct Connection {}

/// This function handles each connection.
async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("New connection: {}", addr);
    // TODO: Maybe have a bigger/dynamic buffer?
    let mut buf = [0; BUFFER_SIZE];

    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            println!("Connection closed: {}", addr);
            return Ok(());
        }

        let response = handle_packet(&buf[..n]).await;
        socket.write_all(&response).await?;
    }
}

/// This function takes the buffer, an array of bytes,
async fn handle_packet(buffer: &[u8]) -> Vec<u8> {
    let packet = Packet::new(buffer);
    print::blue(&format!("NEW PACKET ({}): {}", packet.len(), packet));

    let packet_id = packet.get_id().as_ref().expect("Could not get packet ID!");
    println!("PACKET ID: {}", packet_id.get_value());

    // create a response

    let mut response = Vec::new();
    response.extend_from_slice(b"Received: ");
    response.extend_from_slice(buffer);

    print!("\n\n\n");
    response
}
