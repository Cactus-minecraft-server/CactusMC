//! This module abstracts away a Minecraft packet, so that it can be used in a simple and
//! standardized way.

pub mod data_types;
pub mod utils;

use core::fmt;
use std::{collections::VecDeque, fmt::Debug, io::Read, vec};

use bytes::BytesMut;
use data_types::varint;
use log::warn;
use thiserror::Error;

// It is true that I could lazily evaluate the length, and Id for more performance but I chose to do it eagerly.

/// An abstraction for a Minecraft packet.
///
/// Structure of a normal uncompressed Packet:
///
/// Length (VarInt): Length of Packet ID + Data
/// Packet ID (VarInt): An ID each packet has
/// Data (Byte Array): Actual data bytes
pub struct Packet {
    /// Length of `id` + `data`
    length: usize,

    /// An ID that each Packet has, varint-decoded.
    id: PacketId,

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
    pub fn new(data: &[u8]) -> Result<Self, PacketError> {
        let parsed = Self::parse_packet(&data)?;
        Ok(Self {
            length: parsed.0,
            id: parsed.1,
            data: data.into(),
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

    /// Copies/Clone (I don't quite know) PacketId from the Packet.
    pub fn get_id(&self) -> PacketId {
        self.id
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
    fn parse_packet(data: &[u8]) -> Result<(usize, PacketId, &[u8]), PacketError> {
        let packet_length: (i32, usize) = varint::read(data).map_err(|e| {
            warn!("Failed to decode packet length: {e}");
            PacketError::LengthDecodingError
        })?;

        // We don't add + 1 because we're dealing with 0-indexing.
        let except_length = &data[packet_length.1..];
        let packet_id: (i32, usize) = varint::read(except_length).map_err(|e| {
            warn!("Failed to decode packet ID: {e}");
            PacketError::IdDecodingError
        })?;

        // So this is essentially "except_length_and_id", the continuation of `except_length`
        let payload = &except_length[packet_id.1..];

        let length_value: usize = packet_length.0.try_into().map_err(|e| {
            warn!("Failed to cast length i32 -> usize: {e}");
            PacketError::LengthDecodingError
        })?;

        let id_obj = PacketId::new(packet_id.0, packet_id.1);

        Ok((length_value, id_obj, payload))
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
            id: PacketId::default(),
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

pub enum PacketType {
    Todo,
}

// TODO: Implement std::Display. Print packet type (if found) and value.

/// id_length is the length in bytes of the Packet ID VarInt.
#[derive(Default, Clone)]
pub struct PacketId {
    id: i32,
    id_varint: Vec<u8>,
    id_length: usize,
}

impl PacketId {
    // Instantiates a new PacketId with given id and id_length.
    // To be clear, id_length is the length in bytes of the VarInt.
    pub fn new(id: i32) -> Self {
        let id_varint = data_types::varint::write(id);

        Self {
            id,
            id_length: id_varint.len(),
            id_varint,
        }
    }

    /// The numerical value of the ID (i32).
    pub fn get_value(&self) -> i32 {
        self.id
    }

    /// The length of the VarInt-encoded ID.
    pub fn len(&self) -> usize {
        self.id_length
    }

    /// The VarInt-encoded ID.
    pub fn get_varint(&self) -> Vec<u8> {
        self.id_varint.clone()
    }

    /// Returns the "type" of the packet. An enum representing what the packet is, like connecting
    /// to the server or opening a container in front of the player.
    ///
    /// We return a `Option<PacketType>` because the packet could be unidentified (Rust already has
    /// Option<T>, so we're not adding a None variant to PacketType.)
    pub fn get_type() -> Option<PacketType> {
        todo!()
    }
}

/// Usage:
/// ```rust
/// let data = [0x7F]; // Example of a single-byte varint
/// let packet = Packet::new(&data);
///
/// let id_result: Result<PacketId, &'static str> = PacketId::try_from(&packet);
/// match id_result {
///     Ok(packet_id) => println!("Packet ID: {}, length: {}", packet_id.id, packet_id.id_length),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
impl TryFrom<&Packet> for PacketId {
    type Error = PacketError;

    fn try_from(value: &Packet) -> Result<Self, Self::Error> {
        // TODO: Show Result error for debug.
        if let Ok((id, _)) = data_types::varint::read(&value.data) {
            Ok(Self::new(id))
        } else {
            Err(PacketError::IdDecodingError)
        }
    }
}

impl TryFrom<&[u8]> for PacketId {
    type Error = PacketError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Ok((id, _)) = data_types::varint::read(value) {
            Ok(Self::new(id))
        } else {
            Err(PacketError::IdDecodingError)
        }
    }
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("Failed to decode the packet id")]
    IdDecodingError,

    #[error("Failed to decode the packet length")]
    LengthDecodingError,

    #[error("Failed to build the packet: {0}")]
    BuildPacket(String),
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a packet
    pub fn build(&self, packet_id: i32) -> Result<Packet, PacketError> {
        let id = PacketId::new(packet_id);

        let mut payload = BytesMut::with_capacity(64);
        for action in &self.actions {
            match action {
                BuildAction::AppendBytes(bytes) => payload.extend_from_slice(bytes),
                BuildAction::AppendVarInt(value) => {
                    let varint = data_types::varint::write(*value);
                    payload.extend_from_slice(&varint);
                }
                BuildAction::AppendString(string) => {
                    let string_bytes = data_types::string::write(string)
                        .map_err(|err| PacketError::BuildPacket(err.to_string()))?;
                    payload.extend_from_slice(&string_bytes);
                }
            }
        }

        let length = id.len() + payload.len();
        let length_varint = data_types::varint::write(length as i32);

        let mut data = BytesMut::with_capacity(length + 10);
        data.extend(length_varint);
        data.extend(id.get_varint());
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

        let mut init_data = varint::write(2048); // Length
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

        let mut init_data: Vec<u8> = varint::write(256);
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

        let id = varint::write(1000);
        let data = &[1, 2, 3];
        let length = varint::write((id.len() + data.len()) as i32);

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
