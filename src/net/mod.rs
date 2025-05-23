//! This module manages the TCP server and how/where the packets are managed/sent.
pub mod packet;
pub mod slp;
use crate::config;
use bytes::BytesMut;
use log::{debug, error, info, warn};
use packet::{Packet, PacketError, Response};
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

    /// Tries to close the connection with the Minecraft client
    async fn close(&self) -> Result<(), std::io::Error> {
        self.socket.lock().await.shutdown().await
    }
}

/// Handles each connection. Receives every packet.
async fn handle_connection(socket: TcpStream) -> Result<(), NetError> {
    debug!("Handling new connection: {socket:?}");

    let connection = Connection::new(socket);

    loop {
        // Read the socket and wait for a packet
        let packet: Packet = connection.read().await?;

        let response: Response = handle_packet(&connection, packet).await?;

        if let Some(packet) = response.get_packet() {
            // TODO: Make sure that sent packets are big endians (data types).
            connection.write(packet).await?;

            if response.does_close_conn() {
                warn!("Sent a packet that will close the connection");
                connection.close().await?;
            }
        } else {
            // Temp warn
            warn!("Got response None. Not sending any packet to the MC client");
        }
    }
}

/// This function returns an appropriate response given the input `buffer` packet data.
async fn handle_packet(conn: &Connection, packet: Packet) -> Result<Response, NetError> {
    debug!("{packet:?} / Conn. state: {:?}", conn.get_state().await);

    // Dispatch packet depending on the current State.
    match conn.get_state().await {
        ConnectionState::Handshake => dispatch::handshake(conn).await,
        ConnectionState::Status => dispatch::status(packet).await,
        ConnectionState::Login => dispatch::login(conn, packet).await,
        ConnectionState::Transfer => dispatch::transfer(conn, packet).await,
    }
}

mod dispatch {
    use super::*;
    use packet::Response;

    pub async fn handshake(conn: &Connection) -> Result<Response, NetError> {
        // Set state to Status
        conn.set_state(ConnectionState::Status).await;

        Ok(Response::new(None))
    }

    pub async fn status(packet: Packet) -> Result<Response, NetError> {
        match packet.get_id().get_value() {
            0x00 => {
                // Got Status Request
                let status_resp_packet = slp::status_response()?;
                let response = Response::new(Some(status_resp_packet));

                Ok(response)
            }
            0x01 => {
                // Got Ping Request (status)
                let ping_request_packet = slp::ping_response(packet)?;
                let response = Response::new(Some(ping_request_packet)).close_conn();

                // We should close the connection after sending this packet.
                // See the 7th step:
                // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol_FAQ#What_does_the_normal_status_ping_sequence_look_like?

                Ok(response)
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

    pub async fn login(conn: &Connection, packet: Packet) -> Result<Response, NetError> {
        todo!()
    }

    pub async fn transfer(conn: &Connection, packet: Packet) -> Result<Response, NetError> {
        todo!()
    }
}
