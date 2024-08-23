//! The entrypoint of the server.

// use std::{path::PathBuf, str::FromStr};

// use consts::filepaths;
// use std::path::Path;
mod config;
mod consts;
mod logging;
mod net;
mod packet;
mod file_folder_parser;

mod slp;
use log::{error, info, warn};

fn main() {
    // A testing function, only in debug mode
    #[cfg(debug_assertions)]
    test();
    let server_properties_write = file_folder_parser::create_server_properties(consts::file_content::SERVER_PROPERTIES)
    logging::init(log::LevelFilter::Debug);

    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down...");
        exit(0);
    })
    .expect("Error setting Ctrl+C handler");

    info!("[ SERVER STARTED ]");

    if let Err(e) = init() {
        error!("Failed to start the server: {e}. \nExiting...");
        exit(-1);
    }

    info!("[ SERVER EXITED ]");
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    //let config_file = config::read(Path::new(consts::filepaths::PROPERTIES))
    //   .expect("Error reading server.properties file");

    // init_slp(config_file) or just instantiate the config file in the init_slp function

    net::listen().map_err(|e| {
        error!("Failed to listen for packets: {e}");
        e
    })?;

    Ok(())
}

#[cfg(debug_assertions)]
/// A test fonction that'll only run in debug-mode. (cargo run) and not (cargo run --release)
fn test() {
    info!("[ BEGIN test() ]");

    info!("Hello, world from test()!");

    info!("[ END test()]");
}

/// Exits the server.
fn exit(code: i32) -> ! {
    if code == 0 {
        info!("[ SERVER EXITED ]");
    } else {
        warn!("[ SERVER EXITED WITH ERROR CODE ({code}) ]");
    }

    std::process::exit(code);
}
