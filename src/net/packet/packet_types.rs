//! A module to parse known packets.

// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.
// TODO: HELPER FUNCTION TO READ BYTES WITH INCLUDED OFFSET.

use super::{
    data_types::{CodecError, DataType, ErrorReason, StringProtocol, UnsignedShort, VarInt},
    Packet, PacketId,
};

#[derive(Debug)]
pub enum NextState {
    Status,
    Login,
    Transfer,
}

impl NextState {
    /// Parses a NextState from a VarInt
    pub fn new(next_state: VarInt) -> Result<Self, CodecError> {
        match next_state.get_value() {
            0x01 => Ok(NextState::Status),
            0x02 => Ok(NextState::Login),
            0x03 => Ok(NextState::Transfer),
            _ => Err(CodecError::Decoding(
                DataType::NextState,
                ErrorReason::UnknownValue,
            )),
        }
    }

    /// Returns the i32 associated value from the current NextState object.
    pub fn get_value(&self) -> i32 {
        match self {
            NextState::Status => 1,
            NextState::Login => 2,
            NextState::Transfer => 3,
        }
    }
}

#[derive(Debug)]
pub struct Handshake {
    id: PacketId,
    protocol_version: VarInt,
    server_address: StringProtocol,
    server_port: UnsignedShort,
    next_state: NextState,
}

impl Handshake {
    /// Tries to parse a Handshake packet from bytes.
    /// Accepts `Packet`.
    pub fn new<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();
        let mut offset: usize = 0;

        let id = {
            let varint = VarInt::from_bytes(data)?;
            PacketId::try_from(varint).map_err(|err| {
                CodecError::Decoding(
                    DataType::Handshake,
                    ErrorReason::InvalidFormat(err.to_string()),
                )
            })?
        };
        offset += id.len();

        let protocol_version = VarInt::from_bytes(&data[offset..])?;
        offset += protocol_version.get_bytes().len();

        let server_address = StringProtocol::from_bytes(&data[offset..])?;
        offset += server_address.get_bytes().len();

        let server_port = UnsignedShort::from_bytes(&data[offset..])?;
        offset += server_port.get_bytes().len();

        let next_state = {
            let varint = VarInt::from_bytes(&data[offset..])?;
            NextState::new(varint)?
        };

        Ok(Self {
            id,
            protocol_version,
            server_address,
            server_port,
            next_state,
        })
    }

    /// Returns a reference to the current `PacketID`.
    pub fn get_id(&self) -> &PacketId {
        &self.id
    }

    /// Returns a reference to the current protocol version. (`VarInt`).
    pub fn get_protocol_version(&self) -> &VarInt {
        &self.protocol_version
    }

    /// Returns a reference to the current `StringProtocol`.
    pub fn get_server_address(&self) -> &StringProtocol {
        &self.server_address
    }

    /// Returns a reference to the current server port (`UnsignedShort`).
    pub fn get_server_port(&self) -> &UnsignedShort {
        &self.server_port
    }

    /// Returns a reference to the current `NextState`.
    pub fn get_next_state(&self) -> &NextState {
        &self.next_state
    }
}

impl TryFrom<Packet> for Handshake {
    type Error = CodecError;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        Self::new(value.get_full_packet())
    }
}
