//! The module accountable for making the Server List Ping (SLP) protocol.

// TODO: Encoding/Decoding of VarInts and VarLongs into a separate module, maybe the module packet

// TODO: As disscussed in the main.rs file, we might want to start with running the SLP into a
// separate thread for the sake of simplicity, for now. Then we'll need to dig into concurrency
// rather than pure parallelism.

// TODO: Add logging.

use log::debug;

use super::packet::{PacketBuilder, PacketError};
use crate::consts;
use crate::packet::Packet;

/// The response for a Status Request packet.
pub fn status_response() -> Result<Packet, PacketError> {
    let json_response = consts::protocol::status_response_json();

    PacketBuilder::new()
        .append_string(json_response)
        .build(0x00)
}

/// The response for a Ping Request packet.
pub fn ping_response(ping_request_packet: Packet) -> Result<Packet, PacketError> {
    debug!("Ping packet is: {ping_request_packet}");
    let payload: &[u8] = ping_request_packet.get_payload();
    debug!(
        "Ping packet payload is: {payload:?} and len is {}",
        payload.len()
    );
    if payload.len() == 8 {
        // Send back the same timestamp as what we received
        PacketBuilder::new()
            .append_bytes(&payload[0..8])
            .build(0x01)
    } else {
        Err(PacketError::PayloadDecodeError(
            "failed to decode timestamp (Long) in the Ping Request packet".to_string(),
        ))
    }
}
