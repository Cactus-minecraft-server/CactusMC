//! The servers's entrypoint file.

mod commands;
mod config;
mod consts;
mod file_folder_parser;
mod logging;
mod net;
mod packet;
mod slp;

use colored::Colorize;
use log::{error, info, warn};

#[tokio::main]
async fn main() {
    info!("[ SERVER STARTING... ]");

    if let Err(e) = early_init().await {
        error!("Failed to start the server, error in early initialization: {e}. \nExiting...");
        gracefully_exit(-1);
    }

    if let Err(e) = init() {
        error!("Failed to start the server, error in initialization: {e}. \nExiting...");
        gracefully_exit(-1);
    }

    if let Err(e) = start().await {
        error!("Failed to start the server: {e}. \nExiting...");
        gracefully_exit(-1);
    }

    info!("[ SERVER EXITED ]");
}

/// Logic that must executes as early as possibe
async fn early_init() -> Result<(), Box<dyn std::error::Error>> {
    // This must executes as early as possible
    logging::init(log::LevelFilter::Debug);

    // Adds custom behavior to CTRL + C signal
    init_ctrlc_handler()?;

    // A testing function, only in debug mode
    #[cfg(debug_assertions)]
    test();

    // Listens for cli input commands
    commands::listen_console_commands().await;

    Ok(())
}

/// Essential server initialization logic.
fn init() -> Result<(), Box<dyn std::error::Error>> {
    // Printing a greeting message
    greet();

    // Makes sure server files are initialized and valid.
    file_folder_parser::init()?;

    Ok(())
}

/// Starts up the server.
async fn start() -> Result<(), Box<dyn std::error::Error>> {
    info!("[ SERVER STARTED ]");

    net::listen().await.map_err(|e| {
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
    const GREETINGS: &str = "Hello, world from Copper!";
    info!("{}", GREETINGS.green().bold());
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
pub fn gracefully_exit(code: i32) -> ! {
    // Well, for now it's not "gracefully" exiting.
    if code == 0 {
        info!("[ SERVER EXITED ]");
    } else {
        //warn!("[ SERVER EXITED WITH ERROR CODE ({code}) ]");
    }

    std::process::exit(code);
}
