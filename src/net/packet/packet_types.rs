//! A module to parse known packets.

use core::fmt;
use std::sync::OnceLock;
use super::utils;

use dashmap::DashMap;
use handshake::Handshake;
use log::error;

use super::{
    data_types::{
        Array, CodecError, DataType, Encodable, ErrorReason, StringProtocol, UnsignedShort, Uuid,
        VarInt,
    },
    Packet, PacketBuilder, PacketError,
};

// TODO: MODULES SEPARATING THE PACKET IN THEIR DIFFERENT STATES (HANSHAKE, LOGIN, PLAY,
// CONFIGURATION, ...)

// TODO: If possible: tests using dynamic features to test ALL packets at a time using their
// traits. (Some king of for loop on all Packet types based on GenericPacket, the other traits)

/// Define generic methods that all custom packets should implement.
pub trait GenericPacket: Sized + fmt::Debug + Clone + Eq + PartialEq + Default {
    /// Each packet has its specific ID.
    const PACKET_ID: i32;

    /// Returns a reference to a `Packet` that had been constructed from the current packet type.
    fn get_packet(&self) -> &Packet;

    /// Returns the numer of bytes of the packet.
    fn len(&self) -> usize {
        self.get_packet().len()
    }
}

/// Methods only clienbound packets should implement.
/// Clientbound packets are sent towards the client.
pub trait ClientboundPacket: GenericPacket {
    type PacketFields;

    /// Creates a new packet from values.
    fn from_values(packet_fields: Self::PacketFields) -> Result<Self, PacketError>;
}

/// Methods only serverbound packets should implement.
/// Serverbound packets are sent towards the server (us).
pub trait ServerboundPacket: GenericPacket + TryFrom<Packet> {
    /// Tries to create the object by parsing bytes;
    /// `Packet` is compatible in this function (Because it implements AsRef<[u8]>).
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, PacketError>;

    fn consume_from_bytes(bytes: &mut &[u8]) -> Result<Self, PacketError> {
        let parsed = Self::from_bytes(*bytes)?;
        *bytes = &bytes[parsed.len()..]; // Advance the slice
        Ok(parsed)
    }
}

/// A packet which an empty payload.
pub trait EmptyPayloadPacket: GenericPacket + TryFrom<Packet> {
    /// Returns a packet with no payload.
    fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to a `Packet` that had been constructed from the current packet type.
    ///
    /// All of those shenanigans so that the struct implementing `EmptyPayloadPacket` does not have
    /// to define the `get_packet()` function. This problem could have been fixed by just returning
    /// a `Packet` and not `&Packet` but I want to keep things as they are.
    /// Credit: partly to AI
    fn get_packet(&self) -> &Packet {
        // Thread-safe static HashMap to store OnceLocks for each type
        static PACKET_MAP: OnceLock<DashMap<i32, &'static OnceLock<Packet>>> = OnceLock::new();

        // Initialize the map if not already done
        let map = PACKET_MAP.get_or_init(|| DashMap::new());

        // Retrieve or initialize the OnceLock for this type
        let key: i32 = Self::PACKET_ID;
        let once_lock = map.entry(key).or_insert_with(|| {
            // Box::leak() ensures the OnceLock has a static lifetime
            // (In actuality causes a true memory leak each time it's called)
            Box::leak(Box::new(OnceLock::new()))
        });

        // Initialize the packet if it's not already created
        once_lock.get_or_init(|| {
            let mut packet = Packet::default();
            let packet_id =
                VarInt::from_value(Self::PACKET_ID).expect("PACKET_ID must be a valid VarInt");
            packet.id = packet_id;
            packet
        })
    }

    /// Tries to create the object by parsing bytes;
    /// `Packet` is compatible in this function (Because it implements AsRef<[u8]>).
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, PacketError> {
        let data: &[u8] = bytes.as_ref();
        if !data.is_empty() {
            Err(PacketError::Codec(CodecError::Decoding(
                DataType::Other("Empty payload packet"),
                ErrorReason::InvalidFormat(format!(
                    "The packet payload must be empty. Current is: {data:?}"
                )),
            )))
        } else {
            Ok(Self::new())
        }
    }
}

//Handshake,
//Status,
//Login,
//Configuration,
//Play,
//Transfer,

#[derive(Debug, Clone, PartialEq, Eq)]
/// This is not simply a VarInt, this is an 'Enum VarInt'.
pub enum NextState {
    Handshake(VarInt),
    Status(VarInt),
    Login(VarInt),
    Transfer(VarInt),
}

impl NextState {
    /// Parses a NextState from a VarInt
    pub fn new(next_state: VarInt) -> Result<Self, CodecError> {
        match next_state.get_value() {
            0x00 => Ok(NextState::Handshake(next_state)),
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
            NextState::Handshake(varint) => varint,
            NextState::Status(varint) => varint,
            NextState::Login(varint) => varint,
            NextState::Transfer(varint) => varint,
        }
    }
}

impl Default for NextState {
    fn default() -> Self {
        let varint: VarInt = VarInt::from_value(0).expect("Failed to create VarInt with value=0.");
        NextState::Handshake(varint)
    }
}

pub mod handshake {
    use super::*;

    /// This packet causes the server to switch into the target state, it should be sent right after opening the TCP connection to avoid the server from disconnecting.
    ///
    /// Direction: Serverbound
    /// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Handshake
    #[derive(Debug, Clone, Eq, PartialEq, Default)]
    pub struct Handshake {
        pub protocol_version: VarInt,
        pub server_address: StringProtocol,
        pub server_port: UnsignedShort,
        pub next_state: NextState,

        packet: Packet,
    }

    impl GenericPacket for Handshake {
        const PACKET_ID: i32 = 0x00;

        fn get_packet(&self) -> &Packet {
            &self.packet
        }
    }

    impl ServerboundPacket for Handshake {
        fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, PacketError> {
            let mut data: &[u8] = bytes.as_ref();

            let protocol_version: VarInt = VarInt::consume_from_bytes(&mut data)?;
            let server_address: StringProtocol = StringProtocol::consume_from_bytes(&mut data)?;
            let server_port: UnsignedShort = UnsignedShort::consume_from_bytes(&mut data)?;
            let next_state: NextState = NextState::new(VarInt::consume_from_bytes(&mut data)?)?;

            let packet: Packet = PacketBuilder::new()
                .append_bytes(protocol_version.get_bytes())
                .append_bytes(server_address.get_bytes())
                .append_bytes(server_port.get_bytes())
                .append_bytes(next_state.get_varint().get_bytes())
                .build(Self::PACKET_ID)?;

            println!("1: {}", utils::get_hex_repr(packet.get_payload()));
            println!("2: {}", utils::get_hex_repr(bytes.as_ref()));
            assert_eq!(packet.get_payload(), bytes.as_ref());

            Ok(Self {
                protocol_version,
                server_address,
                server_port,
                next_state,
                packet,
            })
        }
    }

    impl TryFrom<Packet> for Handshake {
        type Error = PacketError;

        fn try_from(value: Packet) -> Result<Self, Self::Error> {
            Self::from_bytes(value.get_payload())
        }
    }
}

pub mod status {}

pub mod login {
    use super::*;

    /// Represents the LoginStart packet.
    /// The second packet in the login sequence.
    ///
    /// Login sequence: https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol_FAQ#What's_the_normal_login_sequence_for_a_client?
    ///
    /// A packet sent by the client to login to the server.
    ///
    /// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Login_Start
    #[derive(Debug, Clone, Default, Eq, PartialEq)]
    pub struct LoginStart {
        pub name: StringProtocol,
        pub player_uuid: Uuid,
        packet: Packet,
    }

    impl GenericPacket for LoginStart {
        const PACKET_ID: i32 = 0x00;

        fn get_packet(&self) -> &Packet {
            &self.packet
        }
    }

    impl ServerboundPacket for LoginStart {
        fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, PacketError> {
            let mut data: &[u8] = bytes.as_ref();

            let name: StringProtocol = StringProtocol::consume_from_bytes(&mut data)?;
            let player_uuid: Uuid = Uuid::consume_from_bytes(&mut data)?;

            let packet: Packet = PacketBuilder::new()
                .append_bytes(name.get_bytes())
                .append_bytes(player_uuid.get_bytes())
                .build(Self::PACKET_ID)?;

            Ok(Self {
                name,
                player_uuid,
                packet,
            })
        }
    }

    impl TryFrom<Packet> for LoginStart {
        type Error = PacketError;

        fn try_from(value: Packet) -> Result<Self, Self::Error> {
            Self::from_bytes(value.get_payload())
        }
    }

    /// Direction: Clientbound
    /// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Login_Success
    #[derive(Debug, Clone, Default, Eq, PartialEq)]
    pub struct LoginSuccess {
        pub uuid: Uuid,
        pub username: StringProtocol,
        pub number_of_properties: VarInt,
        pub property: Vec<Array>, // Each array: [String, String, Boolean, Optional String]
        // There also exists the 'Strict Error Handling' (Boolean) field name which only exists for
        // 1.20.5 to 1.21.1.
        packet: Packet,
    }

    impl GenericPacket for LoginSuccess {
        const PACKET_ID: i32 = 0x02;

        fn get_packet(&self) -> &Packet {
            &self.packet
        }
    }

    impl ClientboundPacket for LoginSuccess {
        type PacketFields = (Uuid, StringProtocol, VarInt, Vec<Array>);
        /// type PacketFields = (Uuid, StringProtocol, VarInt, Array);
        fn from_values(packet_fields: Self::PacketFields) -> Result<Self, PacketError> {
            let property_types: [DataType; 4] = [
                DataType::StringProtocol,
                DataType::StringProtocol,
                DataType::Boolean,
                DataType::Optional(Box::new(DataType::StringProtocol)),
            ];

            for array in &packet_fields.3 {
                if array.get_value() != property_types {
                    return Err(PacketError::BuildPacket(format!(
                        "Incorrect types inside LoginSuccess 'property'. Expected {:?}, got {:?}",
                        property_types,
                        array.get_value()
                    )));
                }
            }

            let packet: Packet = PacketBuilder::new()
                .append_bytes(packet_fields.0.get_bytes())
                .append_bytes(packet_fields.1.get_bytes())
                .append_bytes(packet_fields.2.get_bytes())
                .append_bytes(
                    packet_fields
                        .3
                        .iter()
                        .flat_map(|a| a.get_bytes().iter().copied())
                        .collect::<Vec<_>>(),
                )
                .build(Self::PACKET_ID)?;

            Ok(Self {
                uuid: packet_fields.0,
                username: packet_fields.1,
                number_of_properties: packet_fields.2,
                property: packet_fields.3,
                packet,
            })
        }
    }

    /// Acknowledgement to the Login Success packet sent by the server.
    /// This packet switches the connection state to configuration.
    /// Direction: Serverbound
    /// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Login_Acknowledged
    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    pub struct LoginAcknowledged {}

    impl GenericPacket for LoginAcknowledged {
        const PACKET_ID: i32 = 0x03;

        fn get_packet(&self) -> &Packet {
            EmptyPayloadPacket::get_packet(self)
        }
    }

    impl EmptyPayloadPacket for LoginAcknowledged {}

    impl TryFrom<Packet> for LoginAcknowledged {
        type Error = PacketError;

        fn try_from(value: Packet) -> Result<Self, Self::Error> {
            Self::from_bytes(value.get_payload())
        }
    }
}

pub mod configuration {
    use super::*;

    /// Informs the client of which data packs are present on the server.
    /// Direction: Clientbound
    /// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Clientbound_Known_Packs
    #[derive(Debug, Clone, Default, Eq, PartialEq)]
    pub struct ClientboundKnownPacks {
        /// The number of known packs in the following array
        pub known_pack_count: VarInt,

        /// Array[String (Namespace), String (ID), String (Version)]
        pub known_packs: Vec<Array>,

        packet: Packet,
    }

    impl ClientboundKnownPacks {
        const PACK_DATA_TYPES: [DataType; 3] = [
            DataType::StringProtocol,
            DataType::StringProtocol,
            DataType::StringProtocol,
        ];
    }

    impl GenericPacket for ClientboundKnownPacks {
        const PACKET_ID: i32 = 0x0E;

        fn get_packet(&self) -> &Packet {
            &self.packet
        }
    }

    impl ClientboundPacket for ClientboundKnownPacks {
        type PacketFields = (VarInt, Vec<Array>);

        /// Fields: VarInt, Vec<Array[String, String, String]>
        fn from_values(packet_fields: Self::PacketFields) -> Result<Self, PacketError> {
            let known_pack_count: VarInt = packet_fields.0;
            let known_packs: Vec<Array> = packet_fields.1;

            // Number of packs and actual number of packs not corresponding.
            // Returning an error is just soo loooong
            if known_pack_count.get_value() as usize != known_packs.len() {
                return Err(PacketError::Codec(CodecError::Encoding(
                    DataType::Other("Clientbound Know Packs packet"),
                    ErrorReason::InvalidFormat(format!(
                        "Mismatch: number of packs: {}, actual provided number of packs: {}",
                        known_pack_count.get_value(),
                        known_packs.len()
                    )),
                )));
            }

            // No packs.
            if known_pack_count.get_value() == 0 {
                let packet: Packet = PacketBuilder::new()
                    .append_bytes(known_pack_count.get_bytes())
                    .build(Self::PACKET_ID)?;
                return Ok(Self {
                    known_pack_count,
                    known_packs: Vec::new(),
                    packet,
                });
            }

            // If the layout of the Array is not three StringProtocol.
            for pack in &known_packs {
                for inner_type in pack.get_value() {
                    let cast_type: DataType = (*inner_type).clone().into();
                    if cast_type != DataType::StringProtocol {
                        return Err(
                            PacketError::Codec(
                        CodecError::Encoding(DataType::Other("ClientboundKnownPacks"), ErrorReason::InvalidFormat(
                            format!("The pack array data types must be three consecutive StringProtocol. Pack: {pack:?}")
                        ))
                            )
                    );
                    }
                }
            }

            let packet: Packet = PacketBuilder::new()
                .append_bytes(known_pack_count.get_bytes())
                // Append all bytes from all of the arrays
                .append_bytes(
                    known_packs
                        .iter()
                        .flat_map(|a| a.get_bytes().iter().copied())
                        .collect::<Vec<_>>(),
                )
                .build(Self::PACKET_ID)?;

            Ok(Self {
                known_pack_count,
                known_packs,
                packet,
            })
        }
    }

    /// Informs the server of which data packs are present on the client.
    /// Direction: Serverbound
    /// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Serverbound_Known_Packs
    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    pub struct ServerboundKnownPacks {
        /// The number of known packs in the following array
        pub known_pack_count: VarInt,

        /// Array[String (Namespace), String (ID), String (Version)]
        pub known_packs: Vec<Array>,

        packet: Packet,
    }

    impl GenericPacket for ServerboundKnownPacks {
        const PACKET_ID: i32 = 0x07;

        fn get_packet(&self) -> &Packet {
            &self.packet
        }
    }

    impl ServerboundPacket for ServerboundKnownPacks {
        fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, PacketError> {
            let mut data: &[u8] = bytes.as_ref();

            let known_pack_count: VarInt = VarInt::consume_from_bytes(&mut data)?;
            let pack_count: usize = known_pack_count.get_value() as usize;

            // Define the structure of each known pack once.

            // Parse known packs
            let known_packs: Vec<Array> = (0..pack_count)
                .map(|i| {
                    Array::consume_from_bytes(&mut data, &ClientboundKnownPacks::PACK_DATA_TYPES)
                        .map_err(|e| {
                            CodecError::Decoding(
                                DataType::Array(ClientboundKnownPacks::PACK_DATA_TYPES.to_vec()),
                                ErrorReason::InvalidFormat(format!(
                                    "Failed to parse known pack at index {}. Reason: {e}",
                                    i
                                )),
                            )
                        })
                })
                .collect::<Result<_, _>>()?;

            let packet: Packet = PacketBuilder::new()
                .append_bytes(known_pack_count.get_bytes())
                // Append all bytes from all of the arrays
                .append_bytes(
                    known_packs
                        .iter()
                        .flat_map(|a| a.get_bytes().iter().copied())
                        .collect::<Vec<_>>(),
                )
                .build(Self::PACKET_ID)?;

            Ok(Self {
                known_pack_count,
                known_packs,
                packet,
            })
        }
    }

    impl TryFrom<Packet> for ServerboundKnownPacks {
        type Error = PacketError;

        fn try_from(value: Packet) -> Result<Self, Self::Error> {
            Self::from_bytes(value.get_payload())
        }
    }

    /// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Finish_Configuration
    /// This packet switches the connection state to play
    ///
    /// Direction: Clientbound
    ///
    /// Sent by the server to notify the client that the configuration process has finished.
    /// The client answers with Acknowledge Finish Configuration whenever it is ready to continue.
    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    pub struct FinishConfiguration {}

    impl GenericPacket for FinishConfiguration {
        const PACKET_ID: i32 = 0x03;

        fn get_packet(&self) -> &Packet {
            EmptyPayloadPacket::get_packet(self)
        }
    }

    impl EmptyPayloadPacket for FinishConfiguration {}

    impl TryFrom<Packet> for FinishConfiguration {
        type Error = PacketError;

        fn try_from(value: Packet) -> Result<Self, Self::Error> {
            Self::from_bytes(value.get_payload())
        }
    }
}

pub mod transfer {}

pub mod play {}
