//! The entrypoint of the server.

use std::{path::PathBuf, str::FromStr};

use consts::filepaths;
use std::path::Path;
mod config;
mod consts;
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
    let config_file = config::read(Path::new(consts::filepaths::PROPERTIES))
        .expect("Error reading server.properties file");

    // init_slp(config_file) or just instantiate the config file in the init_slp function
}

#[cfg(debug_assertions)]
/// A test fonction that'll only run in debug-mode. (cargo run) and not (cargo run --release)
fn test() {
    println!("BEGIN test()");

    let p = packet::Packet::new();

    println!("There are {} bytes in the packet p", p.count_bytes());

    println!("END test()");
}

// TODO: How should we run the SLP & the game loop at the same time? That's a tough question,
// I think I'll just avoid the problem by making the SLP run in a separate thread :)

fn init_slp() {
    todo!();
}
