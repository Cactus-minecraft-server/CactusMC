//! This module manages the TCP server and how/where the packets are managed/sent.
pub mod packet;
pub mod slp;
use crate::{config, gracefully_exit};
use bytes::BytesMut;
use log::{debug, error, info, warn};
use packet::data_types::{string, varint, CodecError};
use packet::{Packet, PacketError};
use serde_json::error;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

/// Listening address
/// TODO: Change this. Use config files.
const ADDRESS: &str = "0.0.0.0";

#[derive(Error, Debug)]
pub enum NetError {
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    #[error("Failed to read from socket: {0}")]
    Reading(String),

    #[error("Failed to parse packet: {0}")]
    Parsing(#[from] PacketError),
}

/// Listens for every incoming TCP connection.
pub async fn listen() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Settings::new();
    let server_address = format!("{}:{}", ADDRESS, config.server_port);
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
struct Connection<'a> {
    pub state: ConnectionState,
    socket: &'a mut TcpStream,
}

impl<'a> Connection<'a> {
    fn new(socket: &'a mut TcpStream) -> Self {
        Self {
            state: ConnectionState::default(),
            socket,
        }
    }

    /// Change the state of the current Connection.
    fn set_state(&mut self, new_state: ConnectionState) {
        self.state = new_state
    }
}

/// Handles each connection. Receives every packet.
async fn handle_connection(mut socket: TcpStream, addr: SocketAddr) -> Result<(), NetError> {
    debug!("Handling new connection: {socket:?}");

    // TODO: Maybe find a more efficient packet buffering technique?
    let mut buffer = BytesMut::with_capacity(1024);
    let mut connection = Connection::new(&mut socket);

    loop {
        let read_bytes: usize = connection
            .socket
            .read_buf(&mut buffer)
            .await
            .map_err(|err| NetError::Reading(format!("error reading socket: {err}")))?;

        if read_bytes == 0 {
            debug!("Connection closed gracefully with {addr} (read 0 bytes)");
            return Ok(());
        }

        // TODO: Call a sort of function called "dispatch()" that dispatches the received packet to
        // the appropriate functions.
        // Then, maybe make sort of a "response_queue" that the dispatch functions would add OR
        // just use functional programming and get the response of those dispatch functions.
        // Because, if we use a queue, then how do we set the state of the connection (which is a
        // state-machine)

        let response = handle_packet(&mut connection, &buffer).await?;

        // TODO: Make sure that sent packets are big endians (data types).
        connection.socket.write_all(&response).await?;

        buffer.clear();
    }
}

/// Takes a packet buffer and returns a reponse.
async fn handle_packet<'a>(
    conn: &'a mut Connection<'_>,
    buffer: &[u8],
) -> Result<Vec<u8>, NetError> {
    let packet = Packet::new(buffer)?;
    let packet_id_value: i32 = packet.get_id().get_value();

    debug!(
        "NEW PACKET (Bytes: {} / ID: {} / Conn state: {:?}): {}",
        packet.len(),
        packet_id_value,
        conn.state,
        packet
    );

    // TODO: Implement a fmt::Debug trait for the Packet, such as it prints info like id, ...
    //debug!("PACKET INFO: {packet:?}");

    match packet_id_value {
        0x00 => match conn.state {
            ConnectionState::Handshake => {
                warn!("Handshake packet detected!");
                let next_state = read_handshake_next_state(&packet).await?;
                debug!("next_state is {next_state:?}");
                conn.state = next_state;

                // TODO: CLEANUP THIS MESS. Done hastily to check if it would work (it works!!).

                if let ConnectionState::Status = conn.state {
                    // Send JSON
                    let json = r#"{"version":{"name":"1.21.2","protocol":768},"players":{"max":100,"online":5,"sample":[{"name":"thinkofdeath","id":"4566e69f-c907-48ee-8d71-d7ba5aa00d20"}]},"description":{"text":"Hello, CactusMC!"},"favicon":"data:image/png;base64,<data>","enforcesSecureChat":false}"#;

                    // TODO: Make a packet builder!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

                    let lsp_packet_json = string::write(json)?;
                    let lsp_packet_id: u8 = 0x00;
                    let lsp_packet_len =
                        varint::write((lsp_packet_json.len() + size_of::<u8>()) as i32);

                    let mut lsp_packet: Vec<u8> = Vec::new();
                    lsp_packet.extend_from_slice(&lsp_packet_len);
                    lsp_packet.push(lsp_packet_id);
                    lsp_packet.extend_from_slice(&lsp_packet_json);

                    if let Err(e) = conn.socket.write_all(&lsp_packet).await {
                        error!("Failed to write JSON to client: {e}");
                    }
                }
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
    let mut offset: usize = 0;

    let protocol_version: (i32, usize) = varint::read(data)?;
    offset += protocol_version.1;
    info!("Handshake protocol version received: {protocol_version:?}");

    let server_address: (String, usize) = string::read(&data[offset..])?;
    offset += server_address.1;
    info!("Handshake server address received: {server_address:?}");

    // Read 2 bytes
    let mut slice = &data[offset..offset + 2]; // Create a slice of the two bytes
    let server_port: u16 = byteorder::ReadBytesExt::read_u16::<byteorder::BigEndian>(&mut slice)
        .expect("Unable to read port");
    info!("Handshake server port received: {server_port}");
    offset += 2;

    let next_state: (i32, usize) = varint::read(&data[offset..])?;
    info!("Handshake next state received: {next_state:?}");

    match next_state.1 {
        1 => {
            // 1 is for status
            debug!("Next state from handshake is status (1)");
            Ok(ConnectionState::Status)
        }
        2 => {
            // 2 is for login
            error!("Next state from handshake login (2) not yet supported!");
            gracefully_exit(0);
        }
        _ => {
            error!("Next state from handshake not yet supported!");
            gracefully_exit(0);
        }
    }
}
