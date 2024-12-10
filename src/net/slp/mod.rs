//! The module accountable for making the Server List Ping (SLP) protocol.

// TODO: Encoding/Decoding of VarInts and VarLongs into a separate module, maybe the module packet

// TODO: As disscussed in the main.rs file, we might want to start with running the SLP into a
// separate thread for the sake of simplicity, for now. Then we'll need to dig into concurrency
// rather than pure parallelism.

// TODO: Add logging.

use log::info;

use crate::packet::Packet;

use super::packet::PacketBuilder;

/// The response for a Status Request packet.
pub fn status_response() -> Packet {
    //PacketBuilder::new().set_id(0x01).set_data(data);
}
