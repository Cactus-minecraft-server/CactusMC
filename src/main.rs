//! The servers's entrypoint file.
mod args;
mod commands;
mod config;
mod consts;
mod file_folder_parser;
mod fs_manager;
mod logging;
mod net;
use log::{error, info, warn};
use net::packet;
mod generate_overworld;
mod player;
mod seed_hasher;
mod time;

use config::Gamemode;
use consts::messages;

#[tokio::main]
async fn main() {
    args::init();

    if let Err(e) = early_init().await {
        error!("Failed to start the server, error in early initialization: {e}. \nExiting...");
        gracefully_exit(ExitCode::Failure);
    }

    if let Err(e) = init() {
        error!("Failed to start the server, error in initialization: {e}. \nExiting...");
        gracefully_exit(ExitCode::Failure);
    }

    if let Err(e) = start().await {
        error!("Failed to start the server: {e}. \nExiting...");
        gracefully_exit(ExitCode::Failure);
    }

    info!("{}", *messages::SERVER_SHUTDOWN_SUCCESS);
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

    // TODO: Not sure this has to be in main.rs
    let gamemode1 = match config::Settings::new().gamemode {
        Gamemode::Survival => "Survival",
        Gamemode::Adventure => "Adventure",
        Gamemode::Creative => "Creative",
        Gamemode::Spectator => "Spectator",
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
        gracefully_exit(ExitCode::CtrlC);
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
    // Uh, why is that so?
    let _ = packet::Packet::new(&[]);

    info!("[ END test()]");
}

/// Enum representing standardized server exit codes.
pub enum ExitCode {
    Success,
    Failure,
    CtrlC,
}

/// Gracefully exits the server with an exit code.
pub fn gracefully_exit(exit_code: ExitCode) -> ! {
    let numerical_exit_code: i32 = match exit_code {
        ExitCode::Success => {
            info!("{}", *messages::SERVER_SHUTDOWN_SUCCESS);
            // 0 means success
            0
        }
        ExitCode::Failure => {
            warn!("{}", *messages::SERVER_SHUTDOWN_ERROR);
            // 1 mean general error
            1
        }
        ExitCode::CtrlC => {
            info!("{}", *messages::SERVER_SHUTDOWN_CTRL_C);
            // 130 mean script terminated by Ctrl+C
            130
        }
    };

    // Well, for now it's not "gracefully" exiting.
    std::process::exit(numerical_exit_code);
}
