//! This module abstracts away a Minecraft packet, so that it can be used in a simple and
//! standardized way.

pub mod data_types;
pub mod packet_types;
pub mod utils;

use core::fmt;
use std::{collections::VecDeque, fmt::Debug};

use bytes::BytesMut;
use data_types::{CodecError, Encodable, StringProtocol, VarInt};
use thiserror::Error;

// It is true that I could lazily evaluate the length, and Id for more performance but I chose to do it eagerly.

/// An abstraction for a Minecraft packet.
///
/// Structure of a normal uncompressed Packet:
///
/// Length (VarInt): Length of Packet ID + Data
/// Packet ID (VarInt): An ID each packet has
/// Data (Byte Array): Actual data bytes
#[derive(Clone)]
pub struct Packet {
    /// Length of `id` + `data`
    length: usize,

    /// An ID that each Packet has, varint-decoded.
    id: VarInt,

    /// The raw bytes making the packet. (so it contains ALL of the packet, Length, Packet ID and
    /// the data bytes)
    data: BytesMut,

    /// The raw bytes making the PAYLOAD of the packet. (so this slice does not contain the length
    /// and acket ID)
    payload: BytesMut,
}

// TODO: Implement printing functions to see the bytes in hexadecimal in order and in the reverse
// order.

// TODO: Implement `Iterator` trait to iterate over the packet's bytes in order to then implement
// encoding/decoding functions for VarInts and such.

// TODO: A PACKET BUILDER!!!!!!!!!!!

impl Packet {
    /// Initalizes a new `Packet` by parsing the `data` buffer.
    pub fn new<T: AsRef<[u8]>>(data: T) -> Result<Self, PacketError> {
        let parsed = Self::parse_packet(data.as_ref())?;
        Ok(Self {
            length: parsed.0,
            id: parsed.1,
            data: data.as_ref().into(),
            payload: parsed.2.into(),
        })
    }

    /// This is the WHOLE packet.
    pub fn get_full_packet(&self) -> &[u8] {
        &self.data
    }

    /// This is the PAYLOAD. So the the bytes except the Packet Length and the Packet ID.
    pub fn get_payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns a reference to the packet ID `VarInt`.
    pub fn get_id(&self) -> &VarInt {
        &self.id
    }

    /// Returns the `Packet` `length` attribute. From protocol.
    pub fn get_length(&self) -> usize {
        self.length
    }

    /// Returns the number of bytes in the payload.
    pub fn len_payload(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of bytes bytes in the packet.
    /// To be clear, this is the length of the received TCP packet.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Tries to parse raw bytes and return in order:
    /// (Packet Length, Packet ID, Packet payload bytes)
    fn parse_packet(data: &[u8]) -> Result<(usize, VarInt, &[u8]), PacketError> {
        let packet_len_varint = VarInt::from_bytes(data)?;
        let packet_len_len: usize = packet_len_varint.get_bytes().len();

        // We don't add + 1 because we're dealing with 0-indexing.
        // ALL but the Length (VarInt).
        // ID and Payload.
        let except_length = &data[packet_len_len..];

        let packet_id_varint = VarInt::from_bytes(except_length)?;

        // So this is essentially "except_length_and_id", the continuation of `except_length`
        let payload = &except_length[packet_id_varint.get_bytes().len()..];

        Ok((
            packet_len_varint.get_value() as usize,
            packet_id_varint,
            payload,
        ))
    }
}

/// Allows making a `Packet` object with defaults.
/// Usage:
/// ```rust
/// let packet = Packet::default();
/// ```
impl Default for Packet {
    fn default() -> Self {
        Self {
            length: usize::default(),
            id: VarInt::default(),
            payload: BytesMut::new(),
            data: BytesMut::new(),
        }
    }
}

/// When printing a `Packet`, the hexadecimal representation will be shown.
impl fmt::Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex = utils::get_hex_repr(&self.data);
        write!(f, "{hex}")
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PACKET (Length: {} / ID: {})",
            self.len(),
            self.id.get_value(),
        )
    }
}

impl AsRef<[u8]> for Packet {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("Failed to decode the packet length")]
    LengthDecodingError,

    #[error("Failed to build the packet: {0}")]
    BuildPacket(String),

    #[error("Failed to decode from the payload: {0}")]
    PayloadDecodeError(String),

    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
}

/// Represents the different actions that the PacketBuilder will do to construct the packet payload.
pub enum BuildAction {
    /// Appends raw bytes to the packet payload.
    AppendBytes(Vec<u8>),

    /// Appends an integer as a VarInt to the packet payload.
    AppendVarInt(i32),

    /// Appends a UTF-8 string to the packet payload.
    AppendString(String),
}

/// A builder to build a packet.
#[derive(Default)]
pub struct PacketBuilder {
    /// Queue of actions to process
    actions: VecDeque<BuildAction>,
}

impl PacketBuilder {
    /// Returns an empty Self, Self::default().
    ///
    /// Then use chained method calls to build your `Packet`.
    ///
    /// # Example
    ///
    /// let packet: Packet = PacketBuilder::new().append_varint(3).build()?;
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a packet
    pub fn build(&self, packet_id: i32) -> Result<Packet, PacketError> {
        let id = VarInt::from_value(packet_id)?;

        let mut payload = BytesMut::with_capacity(64);
        for action in &self.actions {
            match action {
                BuildAction::AppendBytes(bytes) => payload.extend_from_slice(bytes),
                BuildAction::AppendVarInt(value) => {
                    let varint = VarInt::from_value(*value)?;
                    payload.extend_from_slice(varint.get_bytes());
                }
                BuildAction::AppendString(string) => {
                    let string_protocol = StringProtocol::from_value(string.clone())?;
                    payload.extend_from_slice(string_protocol.get_bytes());
                }
            }
        }

        let length = id.len() + payload.len();
        let length_varint = VarInt::from_value(length as i32)?;

        // Future self: Why "+ 10"?
        let mut data = BytesMut::with_capacity(length + 10);
        data.extend(length_varint.get_bytes());
        data.extend(id.get_bytes());
        data.extend_from_slice(&payload);

        Ok(Packet {
            length,
            id,
            data,
            payload,
        })
    }

    /// Appends bytes to the back of the packet payload.
    pub fn append_bytes<T: AsRef<[u8]>>(&mut self, data: T) -> &mut Self {
        self.actions
            .push_back(BuildAction::AppendBytes(data.as_ref().to_vec()));
        self
    }

    /// Appends `value` as a VarInt to the back of the packet payload.
    pub fn append_varint(&mut self, value: i32) -> &mut Self {
        self.actions.push_back(BuildAction::AppendVarInt(value));
        self
    }

    /// Appends `string` as a String to the back of the packet payload.
    pub fn append_string<T: AsRef<str>>(&mut self, string: T) -> &mut Self {
        self.actions
            .push_back(BuildAction::AppendString(string.as_ref().to_string()));
        self
    }
}

// TODO: I wonder if having "invalid" value, like a too short/long Length should propagate an error
// when creating a Packet.

/// Represents a reponse to the Minecraft client.
pub struct Response {
    /// The packet to respond
    packet: Option<Packet>,
    /// Whether the server should close the connection after sending this response.
    close_after_response: bool,
}

impl Response {
    pub fn new(packet: Option<Packet>) -> Self {
        Self {
            packet,
            close_after_response: false,
        }
    }

    /// Returns a reference to the packet
    pub fn get_packet(&self) -> Option<&Packet> {
        self.packet.as_ref()
    }

    /// Consumes the Response and returns the packet
    pub fn take_packet(self) -> Option<Packet> {
        self.packet
    }

    /// Sets the `close_after_response` to true, which should make the server close the connection
    /// with the Minecraft client after sending this response.
    pub fn close_conn(mut self) -> Self {
        self.close_after_response = true;
        self
    }

    /// Returns whether or not the connection with the Minecraft client should be closed.
    pub fn does_close_conn(&self) -> bool {
        self.close_after_response
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_packet_creation_valid() {
        // Length = ID + Data = 4
        // ID = 4
        // Data = &[1, 2, 3]
        let init_data = &[4, 4, 1, 2, 3];

        let packet: Packet = Packet::new(init_data).expect("Failed to create packet");

        // 1 = 4
        assert_eq!(packet.get_length(), 4);
        assert_eq!(packet.len(), init_data.len());
        assert_eq!(packet.get_id().get_value(), 4);
        assert_eq!(packet.get_payload(), &[1, 2, 3]);
        assert_eq!(packet.get_full_packet(), init_data);
    }

    #[test]
    fn test_packet_creation_invalid_length_too_short() {
        // Length = 1
        // ID = 4
        // Data = &[1, 2, 3]
        let init_data = &[1, 4, 1, 2, 3];

        let packet: Packet = Packet::new(init_data).expect("Failed to create packet");

        assert_eq!(packet.get_length(), 1);
        assert_eq!(packet.len(), init_data.len());
        assert_eq!(packet.get_id().get_value(), 4);
        assert_eq!(packet.get_payload(), &[1, 2, 3]);
        assert_eq!(packet.get_full_packet(), init_data);
    }

    #[test]
    fn test_packet_creation_invalid_length_too_long() {
        // Length = 2048
        // ID = 4
        // Data = &[1, 2, 3]

        let mut init_data: Vec<u8> = VarInt::from_value(2048).unwrap().get_bytes().to_vec(); // Length
        init_data.push(4); // ID
        init_data.push(1); // Data
        init_data.push(2);
        init_data.push(3);

        let packet: Packet = Packet::new(&init_data).expect("Failed to create packet");

        assert_eq!(packet.get_length(), 2048);
        assert_eq!(packet.get_id().get_value(), 4);
        assert_eq!(packet.get_payload(), &[1, 2, 3]);

        assert_eq!(packet.get_full_packet(), init_data);
        assert_eq!(packet.len(), init_data.len());
    }

    #[test]
    fn test_packet_creation_valid_varint_length() {
        // Length = 256
        // ID = 4
        // Data = &[255; u8], 255 because it's + 1 with the ID

        let mut init_data: Vec<u8> = VarInt::from_value(256).unwrap().get_bytes().to_vec(); // Length
        init_data.push(4);
        let data: &[u8] = &(1..=255).collect::<Vec<u8>>()[..];
        init_data.extend(data);

        let packet: Packet = Packet::new(&init_data).expect("Failed to create packet");

        assert_eq!(packet.get_length(), 256);
        assert_eq!(packet.get_id().get_value(), 4);
        assert_eq!(packet.get_payload(), data);

        assert_eq!(packet.get_full_packet(), init_data);
        assert_eq!(packet.len(), init_data.len());
    }

    #[test]
    fn test_packet_creation_valid_varint_length_id() {
        // Length = ID(varint) + Data = ?
        // ID = 1000
        // Data = &[1, 2, 3]

        let id: Vec<u8> = VarInt::from_value(1000).unwrap().get_bytes().to_vec(); // Length
        let data = &[1, 2, 3];
        let length = VarInt::from_value((id.len() + data.len()) as i32)
            .unwrap()
            .get_bytes()
            .to_vec();

        let mut init_data = Vec::new();
        init_data.extend(length);
        init_data.extend(&id);
        init_data.extend(data);

        let packet: Packet = Packet::new(&init_data).expect("Failed to create packet");

        assert_eq!(packet.get_length(), id.len() + data.len());
        assert_eq!(packet.get_id().get_value(), 1000);
        assert_eq!(packet.get_payload(), data);

        assert_eq!(packet.get_full_packet(), init_data);
        assert_eq!(packet.len(), init_data.len());
    }
}
