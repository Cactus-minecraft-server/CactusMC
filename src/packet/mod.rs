//! This module abstracts away a Minecraft packet, so that it can be used in a simple and
//! standardized way.

pub mod codec;
pub mod utils;

use core::fmt;

use thiserror::Error;

// TODO: contains abstraction over a Minecraft packet. And helper functions in adjacent files like
// VarInt, VarLong, String, ... encoding.

/// An abstraction for a Minecraft packet.
pub struct Packet<'a> {
    length: usize,
    /// An ID that each Packet has, varint-decoded.
    id: Result<PacketId, PacketError>,
    /// The raw bytes making the packet.
    data: &'a [u8],
}

// TODO: Implement some functions or overload the `new()` method to initialize the packet with
// actual bytes.

// TODO: Implement printing functions to see the bytes in hexadecimal in order and in the reverse
// order.

// TODO: Implement `Iterator` trait to iterate over the packet's bytes in order to then implement
// encoding/decoding functions for VarInts and such.

impl<'a> Packet<'a> {
    /// Initalizes a new `Packet` with an empty `data` buffer.
    pub fn new(data: &'a [u8], length: usize) -> Self {
        Self {
            length,
            id: PacketId::try_from(data),
            data,
        }
    }

    /// Returns a reference to the packet `data`.
    pub fn get_data(&self) -> &[u8] {
        self.data
    }

    /// Returns a reference to the packet `id`.
    pub fn get_id(&self) -> &Result<PacketId, PacketError> {
        &self.id
    }

    /// Returns the number of elements inside `data`.
    pub fn len(&self) -> usize {
        self.length
    }

    pub fn len_data(&self) -> usize {
        self.data.len()
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
            length: 0,
            id: Ok(PacketId::default()),
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

pub struct PacketId {
    id: i32,
    id_length: usize,
}

impl PacketId {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// Returns the length in bytes of the id.
    /// Due to VarInt encoding, ids larger than 0x7F (127) will have a length greater than 1 byte.
    /// Values are encoded in 7-bit chunks, with the 8th bit indicating whether another byte follows.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let large_id = PacketId::new(0x80); // 128, which is > 127
    /// assert!(large_id.len() > 1);
    ///
    /// let small_id = PacketId::new(0x7F); // 127
    /// assert_eq!(small_id.len(), 1);
    /// ```
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
        if let Some((id, id_length)) = codec::decode::varint(&value.data) {
            Ok(Self { id, id_length })
        } else {
            Err(PacketError::IdDecodingError)
        }
    }
}

impl TryFrom<&[u8]> for PacketId {
    type Error = PacketError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Some((id, id_length)) = codec::decode::varint(&value) {
            Ok(Self { id, id_length })
        } else {
            Err(PacketError::IdDecodingError)
        }
    }
}

impl Default for PacketId {
    fn default() -> Self {
        Self {
            id: 0,
            id_length: 0,
        }
    }
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("Failed to decode the packet id")]
    IdDecodingError,
}

// TODO: continue tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation() {
        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let length = 4;

        let packet = Packet::new(&data, length);

        let data = [0; 1024];
        let length = 1024;

        assert_eq!(packet.data, data);
    }
}
