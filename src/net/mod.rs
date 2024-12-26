//! This module manages the TCP server and how/where the packets are managed/sent.
pub mod packet;
pub mod slp;
use crate::config;
use bytes::BytesMut;
use log::{debug, error, info, warn};
use packet::data_types::{CodecError, Encodable};
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

    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),

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
    Configuration,
    Play,
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
        debug!(
            "Connection state: {:?} -> {:?}",
            self.get_state().await,
            new_state
        );
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
            debug!("Sent a packet with ID={:02X}", packet.get_id().get_value());

            if response.does_close_conn() {
                warn!("Sent a packet that will close the connection");
                connection.close().await?;
                return Ok(());
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
        ConnectionState::Handshake => dispatch::handshake(packet, conn).await,
        ConnectionState::Status => dispatch::status(packet).await,
        ConnectionState::Login => dispatch::login(conn, packet).await,
        ConnectionState::Configuration => dispatch::configuration(conn, packet).await,
        ConnectionState::Play => dispatch::play(conn, packet).await,
        ConnectionState::Transfer => dispatch::transfer(conn, packet).await,
    }
}

mod dispatch {
    use std::env::args;

    use super::*;
    use packet::packet_types::configuration::*;
    use packet::packet_types::handshake::*;
    use packet::packet_types::login::*;
    use packet::packet_types::status::*;
    use packet::packet_types::ClientboundPacket;
    use packet::packet_types::EmptyPayloadPacket;
    use packet::packet_types::GenericPacket;
    use packet::{
        data_types::{Array, Encodable, VarInt},
        Response,
    };

    pub async fn handshake(packet: Packet, conn: &Connection) -> Result<Response, NetError> {
        // Set state to Status
        debug!("Hanshake packet: {:?}", packet.get_full_packet());
        conn.set_state(ConnectionState::Status).await;

        let handshake_packet: Handshake = packet.try_into()?;

        match handshake_packet.next_state {
            packet::packet_types::NextState::Status(_) => {
                conn.set_state(ConnectionState::Status).await
            }
            packet::packet_types::NextState::Login(_) => {
                conn.set_state(ConnectionState::Login).await
            }
            packet::packet_types::NextState::Transfer(_) => {
                conn.set_state(ConnectionState::Transfer).await
            }
            packet::packet_types::NextState::Handshake(_) => {
                return Err(NetError::Codec(CodecError::Decoding(
                    packet::data_types::DataType::Other("Handshake packet"),
                    packet::data_types::ErrorReason::UnknownValue(
                        "Got handshake next_state.".to_string(),
                    ),
                )));
            }
        }

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
        match packet.get_id().get_value() {
            0x00 => {
                let login_start: LoginStart = packet.try_into()?;

                debug!("Got login Start packet: {login_start:#?}");

                let args = (
                    login_start.player_uuid,
                    login_start.name,
                    VarInt::from_value(0)?, // No packs
                    Vec::new(),
                );

                let login_success: LoginSuccess = LoginSuccess::from_values(args)?;
                Ok(Response::new(Some(login_success.get_packet().clone())))
            }
            0x03 => {
                // Parse it just to be sure it's the Login Acknowledged packet.
                let _login_success: LoginAcknowledged = packet.try_into()?;

                // Switch the connection state to Configuration
                conn.set_state(ConnectionState::Configuration).await;

                // Don't respond anything
                //Ok(Response::new(None))
                //
                // Respond with a Clientbound Known Packs packet
                let args = (VarInt::from_value(0)?, Vec::new());
                let clientbound_known_packs: Packet = ClientboundKnownPacks::from_values(args)?
                    .get_packet()
                    .clone();

                Ok(Response::new(Some(clientbound_known_packs)))
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

    pub async fn transfer(conn: &Connection, packet: Packet) -> Result<Response, NetError> {
        todo!()
    }

    pub async fn configuration(conn: &Connection, packet: Packet) -> Result<Response, NetError> {
        match packet.get_id().get_value() {
            0x00 => {
                // TODO: Cookie Request packet
                Ok(Response::new(None))
            }
            0x01 => {
                // TODO: Clientbound Plugin Message packet
                Ok(Response::new(None))
            }
            0x02 => {
                // TODO: Disconnect packet
                Ok(Response::new(None))
            }
            0x03 => {
                let acknowledge_finish_configuration: FinishConfiguration = packet.try_into()?;
                debug!(
                    "Got Acknowledge Finish Configuration: {acknowledge_finish_configuration:?}"
                );

                conn.set_state(ConnectionState::Play).await;

                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                // TODO: Send a 'Login (play)' packet
                //
                // TO IMPLEMENT (for the 'Login (play)' packet):
                //
                // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Type:Identifier
                // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Type:Long
                // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Type:Unsigned_Byte
                // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Type:Byte
                // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Type:Position
                // https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Type:Int

                Ok(Response::new(None))
            }
            0x04 => {
                // TODO: Clientbound Keep Alive packet
                Ok(Response::new(None))
            }
            0x05 => {
                // TODO: Ping packet
                Ok(Response::new(None))
            }
            0x07 => {
                let serverbound_known_packs: ServerboundKnownPacks = packet.try_into()?;
                debug!("Got Serverbound Known Packs packet: {serverbound_known_packs:?}");

                // Switch connection state to Play.
                conn.set_state(ConnectionState::Play).await;

                let finish_configuration = FinishConfiguration::new();
                let finish_config_packet =
                    EmptyPayloadPacket::get_packet(&finish_configuration).clone();
                Ok(Response::new(Some(finish_config_packet)))
            }
            // TODO: And a lot, lot more to follow

            // 0x10 = 16. Last and sixteenth Configuration packet.
            0x10 => {
                // TODO: Server Links
                Ok(Response::new(None))
            }
            _ => {
                warn!("Unknown packet ID, State: Configuration");
                Err(NetError::UnknownPacketId(format!(
                    "unknown packet ID, State: Configuration, PacketId: {}",
                    packet.get_id().get_value()
                )))
            }
        }
    }

    pub async fn play(conn: &Connection, packet: Packet) -> Result<Response, NetError> {
        match packet.get_id().get_value() {
            _ => {
                warn!("Unknown packet ID, State: Play");
                Err(NetError::UnknownPacketId(format!(
                    "unknown packet ID, State: Play, PacketId: {}",
                    packet.get_id().get_value()
                )))
            }
        }
    }
}
