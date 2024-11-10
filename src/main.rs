//! The servers's entrypoint file.
use std::env;
mod args;
mod commands;
mod config;
mod consts;
mod file_folder_parser;
mod fs_manager;
mod logging;
mod net;
mod packet;

mod slp;
use log::{error, info, warn};
mod player;
mod time;

use config::Gamemode;
use consts::messages;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from("-r")) || args.contains(&String::from("--remove_files")) {
        if let Err(e) = fs_manager::clean_files() {
            error!("Failed to clean files: {e}. \nContinuing with server startup...");
        }
    }
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

    info!("{}", *messages::SERVER_SHUTDOWN);
}

/// Logic that must executes as early as possibe
async fn early_init() -> Result<(), Box<dyn std::error::Error>> {
    // This must executes as early as possible
    logging::init(log::LevelFilter::Debug);

    info!("{}", *messages::SERVER_STARTING);

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
    fs_manager::init()?;
    fs_manager::create_dirs();
    fs_manager::create_other_files();
    let gamemode1 = match config::Settings::new().gamemode {
        Gamemode::SURVIVAL => "Survival",
        Gamemode::ADVENTURE => "Adventure",
        Gamemode::CREATIVE => "Creative",
        Gamemode::SPECTATOR => "Spectator",
    };
    info!("Default game type: {}", gamemode1.to_uppercase());

    Ok(())
}

/// Starts up the server.
async fn start() -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Starting Minecraft server on {}:{}",
        match config::Settings::new().server_ip {
            Some(ip) => ip.to_string(),
            None => "*".to_string(),
        },
        config::Settings::new().server_port
    );
    info!("{}", *messages::SERVER_STARTED);

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
    info!("{}", *messages::GREET);
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
    if code == 0 {
        info!("{}", *messages::SERVER_SHUTDOWN);
    } else {
        warn!("{}", messages::server_shutdown_code(code));
    }

    // Well, for now it's not "gracefully" exiting.
    std::process::exit(code);
}
