//! The entrypoint of the server.

// use std::{path::PathBuf, str::FromStr};

// use consts::filepaths;
// use std::path::Path;
mod config;
mod consts;
mod file_folder_parser;
mod logging;
mod net;
mod packet;
mod slp;
use std::io;

use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use file_folder_parser::check_eula;
use log::{debug, error, info, warn};

fn main() {
    // A testing function, only in debug mode
    #[cfg(debug_assertions)]
    test();
    logging::init(log::LevelFilter::Debug);

    info!("[ SERVER STARTED ]");

    if let Err(e) = init() {
        error!("Failed to start the server: {e}. \nExiting...");
        exit(-1);
    }

    info!("[ SERVER EXITED ]");
}

/// Logic that must executes as early as possibe
fn early_init() -> Result<(), Box<dyn std::error::Error>> {
    // This must executes as early as possible
    logging::init(log::LevelFilter::Debug);

    Ok(())
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    init_ctrlc_handler()?;

    greet();

    make_server_properties();

    make_eula()?;

    //let config_file = config::read(Path::new(consts::filepaths::PROPERTIES))
    //   .expect("Error reading server.properties file");

    // init_slp(config_file) or just instantiate the config file in the init_slp function

    net::listen().map_err(|e| {
        error!("Failed to listen for packets: {e}");
        e
    })?;

    Ok(())
}

/// Sets up a behavior when the user executes CTRL + C.
fn init_ctrlc_handler() -> Result<(), Box<dyn std::error::Error>> {
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down...");
        exit(0);
    })?;

    Ok(())
}

/// Prints the starting greetings
fn greet() {
    const GREETINGS: &str = "Hello, world MiFerris!";
    info!("{}", GREETINGS.green().bold());
}

/// If 'server.properties' does not exist, creates the file and populate it with defaults.
fn make_server_properties() -> io::Result<()> {
    // Get time
    let now = Utc::now();
    let local_time: DateTime<Local> = now.with_timezone(&Local); // Convert to local machine time
    let formatted_time = local_time.format("%a %b %d %H:%M:%S %Y").to_string(); // Format the time

    // Create the file
    let _ = file_folder_parser::create_server_properties(
        consts::file_content::SERVER_PROPERTIES,
        consts::filepaths::PROPERTIES,
        &formatted_time,
    )?;

    Ok(())
}

/// If 'eula.txt' does not exist, create the file and populate it with defaults.
fn make_eula() -> io::Result<()> {
    // Get time
    let now = Utc::now();
    let local_time: DateTime<Local> = now.with_timezone(&Local); // Convert to local machine time
    let formatted_time = local_time.format("%a %b %d %H:%M:%S %Y").to_string(); // Format the time

    let _ = file_folder_parser::create_eula(consts::filepaths::EULA, &formatted_time)?;

    if !check_eula(consts::filepaths::EULA) {
        error!("Cannot start the server. You have not agreed to the 'eula.txt'.");
        exit(-1);
    }

    Ok(())
}

#[cfg(debug_assertions)]
/// A test fonction that'll only run in debug-mode. (cargo run) and not (cargo run --release)
fn test() {
    info!("[ BEGIN test() ]");

    info!("Hello, world from test()!");

    info!("[ END test()]");
}

/// Exits the server with an exit code.
fn exit(code: i32) -> ! {
    if code == 0 {
        info!("[ SERVER EXITED ]");
    } else {
        warn!("[ SERVER EXITED WITH ERROR CODE ({code}) ]");
    }

    std::process::exit(code);
}
