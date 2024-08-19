//! The entrypoint of the server.

use std::{path::PathBuf, str::FromStr};

use consts::filepaths;
use std::path::Path;
mod config;
mod consts;
mod net;
mod packet;
mod slp;

use colored::*;

// TODO: Add and setup a loggin module.

fn main() {
    //println!("[ SERVER STARTED ]";
    println!("{}", "[ SERVER STARTED ]".green());
    init();

    #[cfg(debug_assertions)]
    test();

    println!("{}", "[ SERVER EXITED ]".red());
}

fn init() {
    //let config_file = config::read(Path::new(consts::filepaths::PROPERTIES))
    //   .expect("Error reading server.properties file");

    // init_slp(config_file) or just instantiate the config file in the init_slp function
    net::listen();
}

#[cfg(debug_assertions)]
/// A test fonction that'll only run in debug-mode. (cargo run) and not (cargo run --release)
fn test() {
    println!("{}", "\n[ BEGIN test() ]\n".blue());

    let p = packet::Packet::default();

    let val: i32 = 255;
    let varint_encoded: Vec<u8> = packet::codec::encode::varint(val).unwrap();
    let varint_decoded = packet::codec::decode::varint(&varint_encoded).unwrap().0;

    let hex = packet::utils::get_hex_repr(&varint_encoded);
    println!("{} VarInt-encoded is: {}", val, hex);

    println!("{} --VarInt-decoded--> {}", hex, varint_decoded);

    println!("Hello");
    println!("Hello2");

    let a = config::Settings::new();
    println!("{}", a.server_port);

    println!("{}", "\n[ END test() ]\n".blue());
}

// TODO: How should we run the SLP & the game loop at the same time? That's a tough question,
// I think I'll just avoid the problem by making the SLP run in a separate thread :)

fn init_slp() {
    todo!();
}
