//! This module manages the TCP server and how/where the packets are managed/sent.

use crate::config;
use crate::packet::{Packet, PacketId};
use log::{debug, warn};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

/// Global buffer size when allocating a new packet (in bytes).
const BUFFER_SIZE: usize = 1024;

/// Listens for every incoming TCP connection.
pub async fn listen() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Settings::new();
    let server_address = format!("0.0.0.0:{}", config.server_port);
    let listener = TcpListener::bind(server_address).await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                warn!("Error handling connection from {addr}: {e}");
            }
        });
    }
}

/// State of each connection. (e.g.: handshake, play, ...)
enum ConnectionState {
    Handshake,
}

/// Object representing a TCP connection.
struct Connection {}

/// Handles each connection
async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("New connection: {addr}");
    // TODO: Maybe have a bigger/dynamic buffer?
    let mut buf = [0; BUFFER_SIZE];

    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            debug!("Connection closed: {addr}");
            return Ok(());
        }

        let response = handle_packet(&buf[..n]).await?;
        socket.write_all(&response).await?;
    }
}

/// Takes a packet buffer and returns a reponse.
async fn handle_packet(buffer: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    print!("\n\n\n"); // So that each logged packet is clearly visible.

    let packet = Packet::new(buffer)?;
    debug!("NEW PACKET ({}): {}", packet.len(), packet);

    let packet_id: PacketId = packet.get_id();
    debug!("PACKET ID: {}", packet_id.get_value());

    // create a response

    let mut response = Vec::new();
    response.extend_from_slice(b"Received: ");
    response.extend_from_slice(buffer);

    print!("\n\n\n");
    Ok(response)
}
