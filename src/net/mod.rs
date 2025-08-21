//! This module manages the TCP server and how/where the packets are managed/sent.
pub mod packet;
pub mod slp;

use crate::config;
use crate::net::packet::data_types::DataType;
use crate::net::packet::{data_types, utils};
use bytes::{Buf, BytesMut};
use log::{debug, error, info, warn};
use packet::data_types::{CodecError, Encodable};
use packet::{Packet, PacketError, Response};
use std::cmp::min;
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
    buffer: Mutex<BytesMut>,
}

impl Connection {
    /// Base buffer size and the number of bytes we're trying to read from the socket.
    const BUFFER_SIZE: usize = 512;
    const MAX_PACKET_TRIES: usize = 100;

    fn new(socket: TcpStream) -> Self {
        Self {
            state: Arc::new(Mutex::new(ConnectionState::default())),
            socket: Arc::new(Mutex::new(socket)),
            buffer: Mutex::new(BytesMut::with_capacity(Self::BUFFER_SIZE)),
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

    /// Attempts to read a packet size (in bytes, in VarInt).
    /// If yes, return size. (prepending the conn buffer)
    /// If no, append bytes to connection buffer.
    async fn wait_size(&self, bytes: &[u8]) -> Result<(usize, usize), ()> {
        debug!("INTO WAIT_SIZE");
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        let mut buffer = self.buffer.lock().await;
        let buffer_size: usize = buffer.len();
        // OLDEST bytes first.
        let read: Vec<u8> = buffer.iter().chain(bytes).copied().collect();
        debug!("wait_size() buffer: {}", utils::get_dec_repr(&read));

        match data_types::VarInt::from_bytes(read) {
            Ok(size) => {
                debug!("Read VarInt from wait_size(): {}", size.get_value());
                let s = usize::try_from(size.get_value()).map_err(|e| {
                    error!("Error while converting packet size into usize: {e}");
                })?;
                // Left-shift the buffers' items to account for this VarInt.
                buffer.advance(min(buffer_size, size.len()));
                Ok((s, size.len()))
            }
            Err(_) => {
                //buffer.extend_from_slice(bytes);
                Err(())
            }
        }
    }
    /// Wait for the number of bytes wait_size() spew.
    /// If yes, return packet. (prepending the conn buffer)
    /// If no, append bytes to connection buffer.
    async fn wait_packet(&self, bytes: &[u8], size: usize, size_size: usize) -> Result<Packet, ()> {
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: REWRITE THIS WITH ZERO COPY. INEFFICIENT!
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;
        // TODO: ADD CONCRETE ERRORS AND PROPAGATE;

        let mut buffer = self.buffer.lock().await;
        // Size of connection buffer.
        let buffer_size: usize = buffer.len();

        // OLDEST bytes first!
        let read: Vec<u8> = buffer.iter().chain(bytes).copied().collect();

        // Size of only the payload of the frame (ID + Data)
        //let payload_size: usize = size - size_size;
        // Size of ALL the frame.
        let frame_size: usize = size + size_size;

        // The concatenated [CONN BUFF + SOCK READ] is smaller than the frame.
        if (read.len() < frame_size) {
            buffer.extend_from_slice(bytes);
            warn!("Frame size is bigger than cached data");
            debug!("Cached data: {}", utils::get_dec_repr(&read));
            debug!("Frame size: {}", frame_size);
            Err(())
        } else {
            //if (size_size >= size) {
            if 0 == size {
                error!("Size is zero");
                return Err(());
            }

            let frame = &read[..frame_size];
            debug!("Cached data b4 advance: {}", utils::get_dec_repr(&buffer));
            buffer.advance(min(frame_size, buffer_size));
            debug!(
                "Cached data after advance: {}",
                utils::get_dec_repr(&buffer)
            );
            debug!("Frame: {frame:?}");
            match Packet::new(frame) {
                Ok(p) => {
                    if (frame_size < read.len()) {
                        debug!("Cached data b4 extend: {}", utils::get_dec_repr(&buffer));
                        buffer.extend_from_slice(&read[frame_size..]);
                        debug!("Cached data after extend: {}", utils::get_dec_repr(&buffer));
                    }
                    Ok(p)
                }
                Err(e) => {
                    error!("Error while making packet: {e}");
                    Err(())
                }
            }
        }
    }

    /// TODO: CHECK & REWRITE
    /// TODO: CHECK & REWRITE
    /// TODO: CHECK & REWRITE
    /// TODO: CHECK & REWRITE
    /// TODO: CHECK & REWRITE
    async fn read(&self) -> Result<Packet, NetError> {
        let mut socket = self.socket.lock().await;

        loop {
            // Try to parse the VarInt length from the start of the connection buffer.
            let (len_opt, len_len_opt) = {
                let recv = self.buffer.lock().await;

                if recv.is_empty() {
                    (None, None)
                } else {
                    // Parse at most 5 bytes for the VarInt prefix.
                    let to_parse = recv.len().min(5);
                    match data_types::VarInt::from_bytes(recv[..to_parse].to_vec()) {
                        Ok(v) => {
                            let len_usize = usize::try_from(v.get_value()).map_err(|_| {
                                NetError::Reading("Frame length does not fit in usize".to_string())
                            })?;
                            (Some(len_usize), Some(v.len()))
                        }
                        Err(_) => (None, None), // need more bytes for the length
                    }
                }
            };

            if let (Some(len), Some(len_len)) = (len_opt, len_len_opt) {
                if len == 0 {
                    return Err(NetError::Reading("Zero-length frame".to_string()));
                }

                let total = len_len + len;

                // If we already have the full frame, split and parse.
                if {
                    let recv = self.buffer.lock().await;
                    recv.len() >= total
                } {
                    let mut recv = self.buffer.lock().await;
                    let frame = recv.split_to(total);
                    drop(recv);

                    return Packet::new(&frame).map_err(NetError::Parsing);
                }
                // else: we know the size but need more bytes; fall through to read.
            }

            // Read more bytes from the socket into the connection buffer.
            let mut recv = self.buffer.lock().await;
            let n = socket.read_buf(&mut *recv).await?;
            if n == 0 {
                return Err(NetError::ConnectionClosed("read 0 bytes".to_string()));
            }
        }
    }
    //
    // /// Reads the socket for a full valid frame (packet).
    // /// # Behavior
    // /// - Maintains a per-connection buffer.
    // ///
    // /// Zeroth, if connection buffer not empty, read from it.
    // ///
    // /// First, wait for reading a **complete** packet length (VarInt).
    // ///
    // /// Second, waits for reading the complete frame, so it reads the parsed VarInt.
    // ///
    // /// Third, if there is more bytes than what we read as a first frame, we put that inside
    // /// the connection buffer.
    // ///
    // /// Return the frame.
    // async fn read(&self) -> Result<Packet, NetError> {
    //     // TODO: DROP TRIES AND ADD TIMEOUT!!
    //     // TODO: DROP TRIES AND ADD TIMEOUT!!
    //     // TODO: DROP TRIES AND ADD TIMEOUT!!
    //     // TODO: DROP TRIES AND ADD TIMEOUT!!
    //     // TODO: DROP TRIES AND ADD TIMEOUT!!
    //
    //     let mut buffer = BytesMut::with_capacity(Self::BUFFER_SIZE);
    //     let mut socket = self.socket.lock().await;
    //
    //     for try_count in 1..=Self::MAX_PACKET_TRIES {
    //         // 1) Try to parse the length (once).
    //         let size_res = Self::wait_size(self, &buffer).await;
    //         if size_res.is_err() {
    //             let n = socket.read_buf(&mut buffer).await?;
    //             if n == 0 { return Err(NetError::ConnectionClosed("read 0 bytes".into())); }
    //             continue;
    //         }
    //         let (size, size_len) = size_res.unwrap();
    //
    //         // 2) Try to split the frame.
    //         if let Ok(pkt) = Self::wait_packet(self, &buffer, size, size_len).await {
    //             return Ok(pkt);
    //         }
    //
    //         // 3) Need more bytes.
    //         let n = socket.read_buf(&mut buffer).await?;
    //         if n == 0 { return Err(NetError::ConnectionClosed("read 0 bytes".into())); }
    //     }
    //
    //     return Err(NetError::Reading("Failed to delimit".to_string()));
    //
    //
    //
    //     // Try to read a packet, up to MAX_PACKET_TRIES tries (basically socket reads).
    //     for try_count in 1..=Self::MAX_PACKET_TRIES {
    //         let is_conn_buf_empty: bool = { self.buffer.lock().await.is_empty() };
    //
    //         if Self::wait_size(self, &buffer).await.is_err() {
    //             let n = socket.read_buf(&mut buffer).await?;
    //             if n == 0 { return Err(NetError::ConnectionClosed("read 0 bytes".into())); }
    //             continue;
    //         }
    //
    //         // unwrap??
    //         let (size, size_len) = Self::wait_size(self, &buffer).await.unwrap();
    //
    //         // 2) If we don't yet have size_len + size bytes, read more.
    //         if Self::wait_packet(self, &buffer, size, size_len).await.is_err() {
    //             let n = socket.read_buf(&mut buffer).await?;
    //             if n == 0 { return Err(NetError::ConnectionClosed("read 0 bytes".into())); }
    //             continue;
    //         }
    //
    //         // 3) Now we have a full frame.
    //         let pkt = Self::wait_packet(self, &buffer, size, size_len).await.unwrap();
    //         return Ok(pkt);
    //
    //         debug!("Try in read() to split a frame from stream");
    //         if is_conn_buf_empty {
    //             let read: usize = socket.read_buf(&mut buffer).await?;
    //             if read == 0 {
    //                 warn!("Read zero bytes; connection should be closed.");
    //                 return Err(NetError::ConnectionClosed("read 0 bytes".to_string()));
    //             }
    //         }
    //
    //         debug!(
    //             "TCP buffer received ({}): {}",
    //             buffer.len(),
    //             crate::net::packet::utils::get_dec_repr(&buffer)
    //         );
    //         // VarInt Value / VarInt Size
    //         if let Ok(size_varint) = Self::wait_size(self, &buffer).await {
    //             debug!("Got packet with size: {}", size_varint.0);
    //             if let Ok(packet) =
    //                 Self::wait_packet(self, &buffer, size_varint.0, size_varint.1).await
    //             {
    //                 debug!("Split a packet: {packet:?}");
    //                 return Ok(packet);
    //             } else {
    //                 warn!(
    //                     "Failed to delimit packet with current read bytes; try #{}/{}",
    //                     try_count,
    //                     Self::MAX_PACKET_TRIES
    //                 );
    //             }
    //         } else {
    //             warn!(
    //                 "Failed to delimit packet SIZE with current read bytes; try #{}/{}",
    //                 try_count,
    //                 Self::MAX_PACKET_TRIES
    //             );
    //         }
    //     }
    //
    //     Err(NetError::Reading(
    //         "Failed to delimit packet in socket stream".to_string(),
    //     ))
    // }

    /// Tries to close the connection with the Minecraft client
    async fn close(&self) -> Result<(), std::io::Error> {
        info!("Connection closed: {:?}", self.socket);
        self.socket.lock().await.shutdown().await
    }
}

const IS_CRUEL: bool = true;

/// Handles each connection. Receives every packet.
async fn handle_connection(socket: TcpStream) -> Result<(), NetError> {
    debug!("Handling new connection: {socket:?}");

    let connection = Connection::new(socket);

    #[cfg(debug_assertions)]
    let mut packet_count: usize = 0;

    loop {
        debug!("handle_connection() loop");
        // Read the socket and wait for a packet
        let res = connection.read().await;
        if res.is_err() && IS_CRUEL {
            connection.close().await?;
        }
        let packet: Packet = res?;

        #[cfg(debug_assertions)]
        {
            packet_count += 1;
            debug!("Got packet #{}: {}", packet_count, packet);
        }

        let response: Response = handle_packet(&connection, packet).await?;

        if let Some(packet) = response.get_packet() {
            // TODO: Make sure that sent packets are big endians (data types).
            connection.write(packet).await?;
            debug!("Sent a packet with ID={:02X}", packet.get_id().get_value());

            if response.does_close_conn() {
                info!("Connection closed.");
                connection.close().await?;
                return Ok(());
            }
        } else {
            // Temp warn
            error!("THIS MUST BE FIXED/MORE INFO!!! PLEASE PAY ATTENTION TO ME~~~!");
            warn!("Got response None. Not sending any packet to the MC client");
            //connection.close().await?;
        }
    }
}

//async fn socket_transmitter

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
    use super::*;
    use packet::packet_types::configuration::*;
    use packet::packet_types::handshake::*;
    use packet::packet_types::login::*;
    use packet::packet_types::ClientboundPacket;
    use packet::packet_types::EmptyPayloadPacket;
    use packet::packet_types::GenericPacket;
    use packet::{
        data_types::{Encodable, VarInt},
        Response,
    };

    pub async fn handshake(packet: Packet, conn: &Connection) -> Result<Response, NetError> {
        // Set state to Status
        debug!("Handshake packet: {:?}", packet.get_full_packet());
        conn.set_state(ConnectionState::Status).await;

        debug!("Trying to create handshake packet...");
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
