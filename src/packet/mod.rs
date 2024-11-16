//! This module abstracts away a Minecraft packet, so that it can be used in a simple and
//! standardized way.

pub mod data_types;
pub mod utils;

use core::fmt;

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
pub struct Packet<'a> {
    /// Length of `id` + `data`
    length: usize,

    /// An ID that each Packet has, varint-decoded.
    id: PacketId,

    /// The raw bytes making the packet. (so it contains ALL of the packet, Length, Packet ID and
    /// the data bytes)
    data: &'a [u8],

    /// The raw bytes making the PAYLOAD of the packet. (so this slice does not contain the length
    /// and acket ID)
    payload: &'a [u8],
}

// TODO: Implement printing functions to see the bytes in hexadecimal in order and in the reverse
// order.

// TODO: Implement `Iterator` trait to iterate over the packet's bytes in order to then implement
// encoding/decoding functions for VarInts and such.

// TODO: A PACKET BUILDER!!!!!!!!!!!

impl<'a> Packet<'a> {
    /// Initalizes a new `Packet` with an empty `data` buffer.
    pub fn new(data: &'a [u8]) -> Result<Self, PacketError> {
        let parsed = Self::parse_packet(data)?;
        Ok(Self {
            length: parsed.0,
            id: parsed.1,
            data,
            payload: parsed.2,
        })
    }

    /// This is the WHOLE packet.
    pub fn get_full_packet(&self) -> &[u8] {
        self.data
    }

    /// This is the PAYLOAD. So the the bytes except the Packet Length and the Packet ID.
    pub fn get_payload(&self) -> &[u8] {
        self.payload
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
impl<'a> Default for Packet<'a> {
    fn default() -> Self {
        Self {
            length: usize::default(),
            id: PacketId::default(),
            payload: &[],
            data: &[],
        }
    }
}

/// When printing a `Packet`, the hexadecimal representation will be shown.
impl<'a> fmt::Display for Packet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex = utils::get_hex_repr(self.data);
        write!(f, "{hex}")
    }
}

pub enum PacketType {
    Todo,
}

// TODO: Implement std::Display. Print packet type (if found) and value.

/// id_length is the length in bytes of the Packet ID VarInt.
#[derive(Default, Clone, Copy)]
pub struct PacketId {
    id: i32,
    id_length: usize,
}

impl PacketId {
    // Instantiates a new PacketId with given id and id_length.
    // To be clear, id_length is the length in bytes of the VarInt.
    pub fn new(id: i32, id_length: usize) -> Self {
        Self { id, id_length }
    }

    pub fn get_value(&self) -> i32 {
        self.id
    }

    pub fn len(&self) -> usize {
        self.id_length
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
impl TryFrom<&Packet<'_>> for PacketId {
    type Error = PacketError;

    fn try_from(value: &Packet) -> Result<Self, Self::Error> {
        // TODO: Show Result error for debug.
        if let Ok((id, id_length)) = data_types::varint::read(value.data) {
            Ok(Self { id, id_length })
        } else {
            Err(PacketError::IdDecodingError)
        }
    }
}

impl TryFrom<&[u8]> for PacketId {
    type Error = PacketError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Ok((id, id_length)) = data_types::varint::read(value) {
            Ok(Self { id, id_length })
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
