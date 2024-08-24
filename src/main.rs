//! The servers's entrypoint file.

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
use log::{error, info, warn};

fn main() {
    info!("[ SERVER STARTING... ]");

    if let Err(e) = early_init() {
        error!("Failed to start the server, error in early initialization: {e}. \nExiting...");
        gracefully_exit(-1);
    }

    if let Err(e) = init() {
        error!("Failed to start the server, error in initialization: {e}. \nExiting...");
        gracefully_exit(-1);
    }

    if let Err(e) = start() {
        error!("Failed to start the server: {e}. \nExiting...");
        gracefully_exit(-1);
    }

    info!("[ SERVER EXITED ]");
}

/// Logic that must executes as early as possibe
fn early_init() -> Result<(), Box<dyn std::error::Error>> {
    // This must executes as early as possible
    logging::init(log::LevelFilter::Debug);

    // A testing function, only in debug mode
    #[cfg(debug_assertions)]
    test();

    Ok(())
}

/// Essential server initialization logic.
fn init() -> Result<(), Box<dyn std::error::Error>> {
    init_ctrlc_handler()?;

    greet();

    make_server_properties()?;

    make_eula()?;

    Ok(())
}

/// Starts up the server.
fn start() -> Result<(), Box<dyn std::error::Error>> {
    info!("[ SERVER STARTED ]");

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
        gracefully_exit(0);
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
        gracefully_exit(-1);
    }

    Ok(())
}

#[cfg(debug_assertions)]
/// A test fonction that'll only run in debug-mode. (cargo run) and not (cargo run --release)
fn test() {
    info!("[ BEGIN test() ]");

    // Do not remove this line, yet.
    let _ = packet::Packet::new(&[]);

    info!("Hello, world from test()!");

    info!("[ END test()]");
}

/// Gracefully exits the server with an exit code.
fn gracefully_exit(code: i32) -> ! {
    // Well, for now it's not "gracefully" exiting.
    if code == 0 {
        info!("[ SERVER EXITED ]");
    } else {
        warn!("[ SERVER EXITED WITH ERROR CODE ({code}) ]");
    }

    std::process::exit(code);
}
