//! This module manages the TCP server and how/where the packets are managed/sent.
pub mod packet;
pub mod slp;
use crate::{config, gracefully_exit};
use bytes::BytesMut;
use log::{debug, error, info, warn};
use packet::data_types::{string, varint, CodecError};
use packet::{Packet, PacketError};
use std::io;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Listening address
/// TODO: Change this. Use config files.
const ADDRESS: &str = "0.0.0.0";

#[derive(Error, Debug)]
pub enum NetError {
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    #[error("Failed to read from socket: {0}")]
    Reading(String),

    #[error("Failed to write to socket: {0}")]
    Writing(String),

    #[error("Failed to parse packet: {0}")]
    Parsing(#[from] PacketError),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Unknown packet id: {0}")]
    UnknownPacketId(String),
}

/// Listens for every incoming TCP connection.
pub async fn listen() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Settings::new();
    let server_address = format!("{}:{}", ADDRESS, config.server_port);
    let listener = TcpListener::bind(server_address).await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                warn!("Error handling connection from {addr}: {e}");
            }
        });
    }
}

/// State of each connection. (e.g.: handshake, play, ...)
#[derive(Debug, Clone, Copy)]
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
    state: Arc<Mutex<ConnectionState>>,
    socket: Arc<Mutex<TcpStream>>,
}

impl Connection {
    fn new(socket: TcpStream) -> Self {
        Self {
            state: Arc::new(Mutex::new(ConnectionState::default())),
            socket: Arc::new(Mutex::new(socket)),
        }
    }

    /// Get the current state of the connection
    async fn get_state(&self) -> ConnectionState {
        *self.state.lock().await
    }

    /// Change the state of the current Connection.
    async fn set_state(&self, new_state: ConnectionState) {
        *self.state.lock().await = new_state
    }

    /// Writes either a &[u8] to the socket.
    ///
    /// This function can take in `Packet`.
    async fn write<T: AsRef<[u8]>>(&self, data: T) -> Result<(), NetError> {
        let mut socket = self.socket.lock().await;
        Ok(socket.write_all(data.as_ref()).await?)
    }

    async fn read(&self) -> Result<Packet, NetError> {
        let mut buffer = BytesMut::with_capacity(512);
        let mut socket = self.socket.lock().await;

        let read: usize = socket.read_buf(&mut buffer).await?;

        if read == 0 {
            info!("Connection closed gracefully with (read 0 bytes)");
            return Err(NetError::ConnectionClosed("read 0 bytes".to_string()));
        }

        Ok(Packet::new(&buffer)?)
    }
}

/// Handles each connection. Receives every packet.
async fn handle_connection(socket: TcpStream) -> Result<(), NetError> {
    debug!("Handling new connection: {socket:?}");

    let connection = Connection::new(socket);

    loop {
        // Read the socket and wait for a packet
        let packet: Packet = connection.read().await?;

        let response: Option<Packet> = handle_packet(&connection, packet).await?;

        if let Some(packet) = response {
            // TODO: Make sure that sent packets are big endians (data types).
            connection.write(packet).await?;
        } else {
            // Temp warn
            warn!("Got response None. Not sending any packet to the MC client");
        }
    }
}

/// This function returns an appropriate response given the input `buffer` packet data.
async fn handle_packet(conn: &Connection, packet: Packet) -> Result<Option<Packet>, NetError> {
    debug!("{packet:?} / Conn. state: {:?}", conn.get_state().await);

    // Dispatch packet depending on the current State.
    match conn.get_state().await {
        ConnectionState::Handshake => dispatch::handshake(conn, packet).await,
        ConnectionState::Status => dispatch::status(conn, packet).await,
        ConnectionState::Login => dispatch::login(conn, packet).await,
        ConnectionState::Transfer => dispatch::transfer(conn, packet).await,
    }
}

mod dispatch {
    use super::*;

    pub async fn handshake(conn: &Connection, packet: Packet) -> Result<Option<Packet>, NetError> {
        // Set state to Status
        conn.set_state(ConnectionState::Status).await;

        Ok(None)
    }

    pub async fn status(conn: &Connection, packet: Packet) -> Result<Option<Packet>, NetError> {
        match packet.get_id().get_value() {
            0x00 => {
                // Got Status Request
                let status_resp_packet = slp::status_response()?;
                Ok(Some(status_resp_packet))
            }
            0x01 => {
                // Got Ping Request (status)
                let ping_request_packet = slp::ping_response(packet)?;
                // TODO: WE NEED TO CLOSE THE CONNECTION AFTER SENDING THE PING RESPONSE PACKET.
                // SOURCE: https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol_FAQ#What_does_the_normal_status_ping_sequence_look_like?
                Ok(Some(ping_request_packet))
            }
            _ => {
                warn!("Unknown packet ID, State: Status");
                Err(NetError::UnknownPacketId(format!(
                    "unknown packet ID, State: Status, PacketId: {}",
                    packet.get_id().get_value()
                )))
            }
        }
    }

    pub async fn login(conn: &Connection, packet: Packet) -> Result<Option<Packet>, NetError> {
        todo!()
    }

    pub async fn transfer(conn: &Connection, packet: Packet) -> Result<Option<Packet>, NetError> {
        todo!()
    }
}

//match packet_id_value {
//    0x00 => match conn.state {
//        ConnectionState::Handshake => {
//            warn!("Handshake packet detected!");
//            let next_state = read_handshake_next_state(&packet).await?;
//            debug!("next_state is {next_state:?}");
//            conn.state = next_state;
//
//            // TODO: CLEANUP THIS MESS. Done hastily to check if it would work (it works!!).
//
//            if let ConnectionState::Status = conn.state {
//                // Send JSON
//                let json = r#"{"version":{"name":"1.21.2","protocol":768},"players":{"max":100,"online":5,"sample":[{"name":"thinkofdeath","id":"4566e69f-c907-48ee-8d71-d7ba5aa00d20"}]},"description":{"text":"Hello, CactusMC!"},"favicon":"data:image/png;base64,<data>","enforcesSecureChat":false}"#;
//
//                // TODO: Make a packet builder!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
//
//                let lsp_packet_json = string::write(json)?;
//                let lsp_packet_id: u8 = 0x00;
//                let lsp_packet_len =
//                    varint::write((lsp_packet_json.len() + size_of::<u8>()) as i32);
//
//                let mut lsp_packet: Vec<u8> = Vec::new();
//                lsp_packet.extend_from_slice(&lsp_packet_len);
//                lsp_packet.push(lsp_packet_id);
//                lsp_packet.extend_from_slice(&lsp_packet_json);
//
//                if let Err(e) = conn.socket.write_all(&lsp_packet).await {
//                    error!("Failed to write JSON to client: {e}");
//                }
//            }
//        }
//        _ => {
//            warn!("packet id is 0x00 but State is not yet supported");
//        }
//    },
//    _ => {
//        warn!("Packet ID (0x{packet_id_value:X}) not yet supported.");
//    }
//}
//
//// create a response
//
//let mut response = Vec::new();
//response.extend_from_slice(b"Received: ");
//response.extend_from_slice(buffer);
//
//print!("\n\n\n");
//Ok(response)

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
