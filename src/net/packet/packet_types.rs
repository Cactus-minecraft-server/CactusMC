//! A module to parse known packets.

use super::{
    data_types::{
        CodecError, DataType, Encodable, ErrorReason, StringProtocol, UnsignedShort, VarInt,
    },
    Packet, PacketBuilder, PacketError,
};

#[derive(Debug)]
/// This is not simply a VarInt, this is an Enum VarInt.
pub enum NextState {
    Status(VarInt),
    Login(VarInt),
    Transfer(VarInt),
}

impl NextState {
    /// Parses a NextState from a VarInt
    pub fn new(next_state: VarInt) -> Result<Self, CodecError> {
        match next_state.get_value() {
            0x01 => Ok(NextState::Status(next_state)),
            0x02 => Ok(NextState::Login(next_state)),
            0x03 => Ok(NextState::Transfer(next_state)),
            _ => Err(CodecError::Decoding(
                DataType::Other("NextState"),
                ErrorReason::UnknownValue,
            )),
        }
    }

    /// Returns a reference to the inner VarInt.
    pub fn get_varint(&self) -> &VarInt {
        match self {
            NextState::Status(varint) => varint,
            NextState::Login(varint) => varint,
            NextState::Transfer(varint) => varint,
        }
    }
}

/// Typically a trait only implemented for client-exclusive packets (like Handshake) that the
/// server does not need to create from values, only read from bytes.
///
/// A trait that parses a type of packet from bytes.
trait ParsablePacket: Sized {
    const PACKET_ID: i32;

    /// Tries to create the object by parsing bytes;
    /// `Packet` is compatible in this function (Because it has implemented AsRef).
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError>;

    /// Maybe &Packet, or Packet, or Result<Packet, SomeError>.
    type PacketType;

    /// Returns a **newly created** (inefficient) owned `Packet` from the current packet fields.
    ///
    /// If you have it, use the `Packet` that's already been created because this function creates
    /// a new bytes buffer and then a `Packet`.
    fn get_packet(&self) -> Self::PacketType;

    /// Returns the numer of bytes of the packet.
    fn len(&self) -> usize;
}

/// A trait that allows to encode a type of packet.
trait EncodablePacket: ParsablePacket {
    type Fields;
    fn from_values(packet_fields: Self::Fields) -> Result<Self, CodecError>;
}

#[derive(Debug)]
pub struct Handshake {
    pub protocol_version: VarInt,
    pub server_address: StringProtocol,
    pub server_port: UnsignedShort,
    pub next_state: NextState,

    /// Number of bytes of the packet
    length: usize,
}

impl ParsablePacket for Handshake {
    const PACKET_ID: i32 = 0x00;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let mut data: &[u8] = bytes.as_ref();

        let protocol_version: VarInt = VarInt::consume_from_bytes(&mut data)?;
        let server_address: StringProtocol = StringProtocol::consume_from_bytes(&mut data)?;
        let server_port: UnsignedShort = UnsignedShort::consume_from_bytes(&mut data)?;
        let next_state: NextState = NextState::new(VarInt::consume_from_bytes(&mut data)?)?;

        let length: usize = protocol_version.len()
            + server_address.len()
            + server_port.len()
            + next_state.get_varint().len();

        Ok(Self {
            protocol_version,
            server_address,
            server_port,
            next_state,
            length,
        })
    }

    type PacketType = Result<Packet, PacketError>;

    fn get_packet(&self) -> Self::PacketType {
        PacketBuilder::new()
            .append_bytes(self.protocol_version.get_bytes())
            .append_bytes(self.server_address.get_bytes())
            .append_bytes(self.server_port.get_bytes())
            .append_bytes(self.next_state.get_varint().get_bytes())
            .build(Self::PACKET_ID)
    }

    fn len(&self) -> usize {
        self.length
    }
}

impl TryFrom<Packet> for Handshake {
    type Error = CodecError;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        Self::from_bytes(value)
    }
}
