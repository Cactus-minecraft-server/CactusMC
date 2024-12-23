//! A module to parse known packets.

// TODO: MODULES SEPARATING THE PACKET IN THEIR DIFFERENT STATES (HANSHAKE, LOGIN, PLAY,
// CONFIGURATION, ...)
//
// TODO: SOME GENERIC PACKET WITH NO FIELDS (it seems their are quite numerous, habing a generic
// one would save some code)

use core::fmt;

use log::{debug, error};

use crate::{gracefully_exit, player};

use super::{
    data_types::{
        Array, CodecError, DataType, DataTypeContent, Encodable, ErrorReason, StringProtocol,
        UnsignedShort, Uuid, VarInt,
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
                ErrorReason::UnknownValue(format!("Got {}.", next_state.get_value())),
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
pub trait ParsablePacket: Sized + fmt::Debug {
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
pub trait EncodablePacket: ParsablePacket {
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

        debug!("data: {data:?}");

        let protocol_version: VarInt = VarInt::consume_from_bytes(&mut data)?;
        debug!("data: {data:?}");

        let server_address: StringProtocol = StringProtocol::consume_from_bytes(&mut data)?;
        debug!("data: {data:?}");

        let server_port: UnsignedShort = UnsignedShort::consume_from_bytes(&mut data)?;
        debug!("data: {data:?}");

        let next_state: NextState = NextState::new(VarInt::consume_from_bytes(&mut data)?)?;
        debug!("data: {data:?}");

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
        Self::from_bytes(value.get_payload())
    }
}

/// Represents the LoginStart packet.
/// The second packet in the login sequence.
///
/// Login sequence: https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol_FAQ#What's_the_normal_login_sequence_for_a_client?
///
/// A packet sent by the client to login to the server.
///
/// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Login_Start
#[derive(Debug)]
pub struct LoginStart {
    pub name: StringProtocol,
    pub player_uuid: Uuid,

    /// The number of bytes of the packet.
    length: usize,
}

impl ParsablePacket for LoginStart {
    const PACKET_ID: i32 = 0x00;

    /// Tries to parse a LoginStart packet from bytes.
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let mut data: &[u8] = bytes.as_ref();

        let name: StringProtocol = StringProtocol::consume_from_bytes(&mut data)?;
        let player_uuid: Uuid = Uuid::consume_from_bytes(&mut data)?;
        let length: usize = name.len() + player_uuid.len();

        Ok(Self {
            name,
            player_uuid,
            length,
        })
    }

    type PacketType = Result<Packet, PacketError>;

    fn get_packet(&self) -> Self::PacketType {
        PacketBuilder::new()
            .append_bytes(self.name.get_bytes())
            .append_bytes(self.player_uuid.get_bytes())
            .build(Self::PACKET_ID)
    }

    fn len(&self) -> usize {
        self.length
    }
}

impl TryFrom<Packet> for LoginStart {
    type Error = CodecError;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        Self::from_bytes(value.get_payload())
    }
}

#[derive(Debug)]
pub struct LoginSuccess {
    uuid: Uuid,
    username: StringProtocol,
    number_of_properties: VarInt,
    property: Array,
    // There also exists the 'Strict Error Handling' (Boolean) field name which only exists for
    // 1.20.5 to 1.21.1.
}

impl ParsablePacket for LoginSuccess {
    const PACKET_ID: i32 = 0x02;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        error!("Tried to parse a server-only packet (Login Success). Closing the server...");
        gracefully_exit(crate::ExitCode::Failure);
    }

    type PacketType = Result<Packet, PacketError>;

    fn get_packet(&self) -> Self::PacketType {
        PacketBuilder::new()
            .append_bytes(self.uuid.get_bytes())
            .append_bytes(self.username.get_bytes())
            .append_bytes(self.number_of_properties.get_bytes())
            .build(Self::PACKET_ID)
    }

    fn len(&self) -> usize {
        self.uuid.len() + self.username.len() + self.number_of_properties.len()
    }
}

impl EncodablePacket for LoginSuccess {
    // UUID (Uuid)
    // Username (StringProtocol)
    // Number of Properties (VarInt)
    // Property (Array[StringProtocol, StringProtocol, Boolean, OptionalStringProtocol])
    type Fields = (Uuid, StringProtocol, VarInt, Array);

    fn from_values(packet_fields: Self::Fields) -> Result<Self, CodecError> {
        Ok(Self {
            uuid: packet_fields.0,
            username: packet_fields.1,
            number_of_properties: packet_fields.2,
            property: packet_fields.3,
        })
    }
}

/// This packet switches the connection state to configuration.
#[derive(Debug)]
pub struct LoginAcknowledged {}

impl ParsablePacket for LoginAcknowledged {
    const PACKET_ID: i32 = 0x03;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        if bytes.as_ref().len() != 0 {
            Err(CodecError::Decoding(
                DataType::Other("Login Acknowledged packet"),
                ErrorReason::InvalidFormat(
                    "The payload of the LoginAcknowledged packet should be empty.".to_string(),
                ),
            ))
        } else {
            Ok(Self {})
        }
    }

    type PacketType = Packet;

    fn get_packet(&self) -> Self::PacketType {
        Packet::default()
    }

    fn len(&self) -> usize {
        0
    }
}

impl TryFrom<Packet> for LoginAcknowledged {
    type Error = CodecError;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        Self::from_bytes(value.get_payload())
    }
}

/// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Clientbound_Known_Packs
#[derive(Debug)]
pub struct ClientboundKnownPacks {
    /// The number of known packs in the following array
    known_pack_count: VarInt,

    /// Array[String (Namespace), String (ID), String (Version)]
    known_packs: Vec<Array>,
}

impl ClientboundKnownPacks {
    const PACK_DATA_TYPES: [DataType; 3] = [
        DataType::StringProtocol,
        DataType::StringProtocol,
        DataType::StringProtocol,
    ];
}

impl ParsablePacket for ClientboundKnownPacks {
    const PACKET_ID: i32 = 0x0E;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let mut data: &[u8] = bytes.as_ref();

        let known_pack_count: VarInt = VarInt::consume_from_bytes(&mut data)?;
        let pack_count: usize = known_pack_count.get_value() as usize;

        // Define the structure of each known pack once.

        // Parse known packs
        let known_packs: Vec<Array> = (0..pack_count)
            .map(|i| {
                Array::consume_from_bytes(&mut data, &Self::PACK_DATA_TYPES).map_err(|e| {
                    CodecError::Decoding(
                        DataType::Array(Self::PACK_DATA_TYPES.to_vec()),
                        ErrorReason::InvalidFormat(format!(
                            "Failed to parse known pack at index {}. Reason: {e}",
                            i
                        )),
                    )
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            known_pack_count,
            known_packs,
        })
    }

    type PacketType = Result<Packet, PacketError>;

    fn get_packet(&self) -> Self::PacketType {
        PacketBuilder::new()
            .append_bytes(self.known_pack_count.get_bytes())
            // Make a single buffer of bytes containing all packs.
            .append_bytes(
                self.known_packs
                    .iter()
                    .flat_map(|pack| pack.get_bytes().iter().copied())
                    .collect::<Vec<u8>>(),
            )
            .build(Self::PACKET_ID)
    }

    fn len(&self) -> usize {
        self.known_pack_count.len() + self.known_packs.len()
    }
}

impl EncodablePacket for ClientboundKnownPacks {
    type Fields = (VarInt, Option<Vec<Array>>);

    fn from_values(packet_fields: Self::Fields) -> Result<Self, CodecError> {
        let known_pack_count: VarInt = packet_fields.0;
        if let None = packet_fields.1 {
            if known_pack_count.get_value() != 0 {
                return Err(CodecError::Encoding(
                    DataType::Other("ClientboundKnownPacks packet"),
                    ErrorReason::InvalidFormat(format!(
                        "Even though known packs is None, the known packet count is {}",
                        known_pack_count.get_value()
                    )),
                ));
            }
            return Ok(Self {
                known_pack_count,
                known_packs: Vec::new(),
            });
        }

        let known_packs: Vec<Array> = packet_fields.1.unwrap();

        // If number of packs is the the actual number of packs.
        if known_pack_count.get_value() as usize != known_packs.len() {
            return Err(CodecError::Encoding(
                DataType::Other("ClientboundKnownPacks packet"),
                ErrorReason::InvalidFormat(format!(
                    "The VarInt value must correspond to the number of packs. VarInt value: {} / Number of packs: {}", known_pack_count.get_value(), known_packs.len()
                )),
            ));
        }

        // If the layout of the Array is not three StringProtocol.
        for pack in &known_packs {
            for inner_type in pack.get_value() {
                let cast_type: DataType = (*inner_type).clone().into();
                if cast_type != DataType::StringProtocol {
                    return Err(
                        CodecError::Encoding(DataType::Other("ClientboundKnownPacks"), ErrorReason::InvalidFormat(
                            format!("The pack array data types must be three consecutive StringProtocol. Pack: {pack:?}")
                        ))
                    );
                }
            }
        }

        Ok(Self {
            known_pack_count,
            known_packs,
        })
    }
}

impl TryFrom<Packet> for ClientboundKnownPacks {
    type Error = CodecError;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        Self::from_bytes(value.get_payload())
    }
}

/// Essentially a wrapper around `ClientboundKnownPacks`, only the ID is different.
#[derive(Debug)]
pub struct ServerboundKnownPacks {
    inner: ClientboundKnownPacks,
}

impl ParsablePacket for ServerboundKnownPacks {
    const PACKET_ID: i32 = 0x07;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let inner = ClientboundKnownPacks::from_bytes(bytes)?;
        Ok(Self { inner })
    }

    type PacketType = Result<Packet, PacketError>;

    fn get_packet(&self) -> Self::PacketType {
        let mut packet = self.inner.get_packet()?;
        packet.id = VarInt::from_value(Self::PACKET_ID)?;
        Ok(packet)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl TryFrom<Packet> for ServerboundKnownPacks {
    type Error = CodecError;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        Self::from_bytes(value.get_payload())
    }
}

#[derive(Debug)]
/// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Finish_Configuration
/// This packet switches the connection state to play
///
/// Sent by the server to notify the client that the configuration process has finished.
/// The client answers with Acknowledge Finish Configuration whenever it is ready to continue.
pub struct FinishConfiguration {}

impl ParsablePacket for FinishConfiguration {
    const PACKET_ID: i32 = 0x03;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        if bytes.as_ref().len() != 0 {
            Err(CodecError::Decoding(
                DataType::Other("Finish Configuration packet"),
                ErrorReason::InvalidFormat(
                    "The payload of the LoginAcknowledged packet should be empty.".to_string(),
                ),
            ))
        } else {
            Ok(Self {})
        }
    }

    type PacketType = Result<Packet, PacketError>;

    fn get_packet(&self) -> Self::PacketType {
        PacketBuilder::new().build(Self::PACKET_ID)
    }

    fn len(&self) -> usize {
        0
    }
}

impl EncodablePacket for FinishConfiguration {
    type Fields = Option<bool>;

    fn from_values(packet_fields: Self::Fields) -> Result<Self, CodecError> {
        Ok(Self {})
    }
}

impl TryFrom<Packet> for FinishConfiguration {
    type Error = CodecError;

    fn try_from(value: Packet) -> Result<Self, Self::Error> {
        Self::from_bytes(value.get_payload())
    }
}
