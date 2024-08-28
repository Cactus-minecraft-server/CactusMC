use std::fs::{self, create_dir, File};
use std::io::{self, BufRead};
use std::path::Path;
mod utils;
use colored::Colorize;
use log::{error, info, warn};

use crate::{consts, gracefully_exit};

// Initializes the server's required files and directories
pub fn init() -> std::io::Result<()> {
    create_server_properties()?;

    eula()
}

/// Checks if the eula is agreed, if not creates it.
fn eula() -> io::Result<()> {
    let path = Path::new(consts::filepaths::EULA);
    if !path.exists() {
        create_eula()?;
        let content = "Please agree to the 'eula.txt' and start the server again.";
        warn!("{}",content.bright_red().bold());
        gracefully_exit(0);
    } else {
        let is_agreed_eula = check_eula()?;
        if !is_agreed_eula {
            let error_content = "Cannot start the server, please agree to the 'eula.txt'";
            error!("{}",error_content.bright_red().bold().blink());
            gracefully_exit(-1);
        }
        Ok(())
    }
}

/// Creates the 'server.properties' file if it does not already exist.
fn create_server_properties() -> io::Result<()> {
    let path = Path::new(consts::filepaths::PROPERTIES);
    let content = consts::file_content::server_properties();

    utils::create_file(&path, &content)
}

/// Creates the 'eula.txt' file if it does not already exist.
fn create_eula() -> io::Result<()> {
    let path = Path::new(consts::filepaths::EULA);
    let content = consts::file_content::eula();

    utils::create_file(&path, &content)
}

/// Check if the 'eula.txt' has been agreed to.
fn check_eula() -> io::Result<bool> {
    let file = File::open(Path::new(consts::filepaths::EULA))?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("eula=") {
            let eula_value = line.split('=').nth(1).unwrap_or("").to_lowercase();
            return Ok(eula_value == "true");
        }
    }

    Ok(false)
}

pub fn clean_file() ->(){
    let eula = consts::filepaths::EULA;
    let server_properties = consts::filepaths::PROPERTIES;
    match fs::remove_file(eula) {
        Ok(_) =>info!("File delete."),
        Err(e) => info!("Error when delete file: {}",e),
    }
    match fs::remove_file(server_properties) {
        Ok(_) =>info!("File delete."),
        Err(e) => info!("Error when delete file: {}",e),
    }
}

pub fn create_other_files() { // @Urpagin i think you'll modify this... DO NOT HESITATE ;)
    utils::create_file_nn(Path::new(consts::filepaths::BANNED_IP));
    info!("Created file{}",consts::filepaths::BANNED_IP);
    utils::create_file_nn(Path::new(consts::filepaths::BANNED_PLAYERS));
    info!("Created file{}",consts::filepaths::BANNED_PLAYERS);
    utils::create_file_nn(Path::new(consts::filepaths::OPERATORS));
    info!("Created file{}",consts::filepaths::OPERATORS);
    utils::create_file_nn(Path::new(consts::filepaths::SESSION));
    info!("Created file{}",consts::filepaths::SESSION);
    utils::create_file_nn(Path::new(consts::filepaths::USERCACHE));
    info!("Created file{}",consts::filepaths::USERCACHE);
    utils::create_file_nn(Path::new(consts::filepaths::VERSION));
    info!("Created file{}",consts::filepaths::VERSION);
    utils::create_file_nn(Path::new(consts::filepaths::WHITELIST));
    info!("Created file{}",consts::filepaths::WHITELIST);
}
pub fn create_dirs(){
    utils::create_dir(Path::new(consts::folderpath::LOGS));
    info!("Created dir{}",consts::folderpath::LOGS);
    utils::create_dir(Path::new(consts::folderpath::WORLDS_DIRECTORY));
    info!("Created dir{}",consts::folderpath::WORLDS_DIRECTORY);
    utils::create_dir(Path::new(consts::folderpath::OVERWORLD));
    info!("Created dir{}",consts::folderpath::OVERWORLD);
    utils::create_dir(Path::new(consts::folderpath::THE_END));
    info!("Created dir{}",consts::folderpath::THE_END);
    utils::create_dir(Path::new(consts::folderpath::NETHER));
    info!("Created dir{}",consts::folderpath::NETHER);
}
