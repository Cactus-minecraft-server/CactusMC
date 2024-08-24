//! The module accountable for making the Server List Ping (SLP) protocol.

// TODO: Encoding/Decoding of VarInts and VarLongs into a separate module, maybe the module packet

// TODO: As disscussed in the main.rs file, we might want to start with running the SLP into a
// separate thread for the sake of simplicity, for now. Then we'll need to dig into concurrency
// rather than pure parallelism.

// TODO: Add logging.

use log::info;

use crate::packet::Packet;

pub fn init() {}

fn thread_loop() {}

pub fn slp_callback(connection: i32, packet: &Packet) {
    info!("In slp_callback()");
    // I guess this function will use `connection` to respond.
}
