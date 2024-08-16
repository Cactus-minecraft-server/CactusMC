//! This module abstracts away a Minecraft packet, so that it can be used in a simple and
//! standardized way.

pub mod codec;
pub mod utils;

// TODO: contains abstraction over a Minecraft packet. And helper functions in adjacent files like
// VarInt, VarLong, String, ... encoding.

/// An abstraction for a Minecraft packet.
pub struct Packet {
    /// The raw bytes making the packet
    data: Vec<u8>,
    // TODO: Study the wiki.vg for information about a packet an its potential attributes.
}

// TODO: Implement some functions or overload the `new()` method to initialize the packet with
// actual bytes.

// TODO: Implement printing functions to see the bytes in hexadecimal in order and in the reverse
// order.

// TODO: Implement `Iterator` trait to iterate over the packet's bytes in order to then implement
// encoding/decoding functions for VarInts and such.

impl Packet {
    /// Initalizes a new `Packet` with an empty `data` buffer.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Returns a reference to `data`
    pub fn get_raw_bytes(&self) -> &Vec<u8> {
        &self.data
    }

    /// A simple "mock" function
    pub fn count_bytes(&self) -> usize {
        self.data.len()
    }
}
