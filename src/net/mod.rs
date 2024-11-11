//! This module manages the TCP server and how/where the packets are managed/sent.

use crate::config;
use crate::packet::data_types::{varint, CodecError};
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
#[derive(Debug)]
enum ConnectionState {
    Handshake,
    Status,
    Login,
    Transfer,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Handshake
    }
}

/// Object representing a TCP connection.
struct Connection {
    state: ConnectionState,
    socket: TcpStream,
}

impl Connection {
    fn new(socket: TcpStream) -> Self {
        Self {
            state: ConnectionState::default(),
            socket,
        }
    }
}

/// Handles each connection
async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("New connection: {addr}");
    // TODO: Maybe have a bigger/dynamic buffer?
    let mut buf = [0; BUFFER_SIZE];
    let mut state = ConnectionState::default();

    loop {
        let read_bytes = socket.read(&mut buf).await?;
        if read_bytes == 0 {
            debug!("Connection closed: {addr}");
            return Ok(()); // TODO: Why Ok? It's supposed to be an error right?
        }

        let response = handle_packet(&mut state, &buf[..read_bytes]).await?;
        socket.write_all(&response).await?;
    }
}

/// Takes a packet buffer and returns a reponse.
async fn handle_packet(
    state: &mut ConnectionState,
    buffer: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    print!("\n\n\n"); // So that each logged packet is clearly visible.

    let packet = Packet::new(buffer)?;
    debug!("NEW PACKET ({}): {}", packet.len(), packet);

    // TODO: Implement a fmt::Debug trait for the Packet, such as it prints info like id, ...
    //debug!("PACKET INFO: {packet:?}");

    let packet_id_value: i32 = packet.get_id().get_value();
    debug!("PACKET ID: {packet_id_value}");

    match packet_id_value {
        0x00 => match state {
            ConnectionState::Handshake => {
                warn!("Handshake packet detected!");
                let next_state = read_handshake_next_state(&packet).await;
                println!("next_state is {next_state:?}");
            }
            _ => {
                warn!("packet id is 0x00 but State is not yet supported");
            }
        },
        _ => {
            warn!("Packet ID (0x{packet_id_value:X}) not yet supported.");
        }
    }

    // create a response

    let mut response = Vec::new();
    response.extend_from_slice(b"Received: ");
    response.extend_from_slice(buffer);

    print!("\n\n\n");
    Ok(response)
}

async fn read_handshake_next_state(packet: &Packet<'_>) -> Result<ConnectionState, CodecError> {
    let data = packet.get_payload();

    let bytes_protocol_version = varint::read(data);

    Ok(ConnectionState::Handshake)
}
