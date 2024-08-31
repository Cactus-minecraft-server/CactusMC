use std::fs::{self, create_dir, File};
use std::io::{self, BufRead};
use std::path::Path;
mod utils;
use colored::Colorize;
use log::{error, info, warn};

use crate::{consts, gracefully_exit};

// Initializes the server's required files and directories
pub fn init() -> std::io::Result<()> {

    eula()?;
    create_server_properties()
}

/// Checks if the eula is agreed, if not creates it.
fn eula() -> io::Result<()> {
    let path = Path::new(consts::filepaths::EULA);
    if !path.exists() {
        create_eula()?;
        let content = "Please agree to the 'eula.txt' and start the server again.";
        warn!("{}", content.bright_red().bold());
        gracefully_exit(0);
    } else {
        let is_agreed_eula = check_eula()?;
        if !is_agreed_eula {
            let error_content = "Cannot start the server, please agree to the 'eula.txt'";
            error!("{}", error_content.bright_red().bold().blink());
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

pub fn clean_file() -> () {
    match fs::remove_file(consts::filepaths::EULA) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_file(consts::filepaths::PROPERTIES) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_file(consts::filepaths::BANNED_IP) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_file(consts::filepaths::BANNED_PLAYERS) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_file(consts::filepaths::OPERATORS) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_file(consts::filepaths::SESSION) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_file(consts::filepaths::USERCACHE) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_file(consts::filepaths::WHITELIST) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    //clean folder:
    match fs::remove_dir(consts::folderpath::LOGS) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_dir(consts::folderpath::NETHER) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_dir(consts::folderpath::OVERWORLD) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_dir(consts::folderpath::THE_END) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
    match fs::remove_dir(consts::folderpath::WORLDS_DIRECTORY) {
        Ok(_) => info!("File delete."),
        Err(e) => info!("Error when delete file: {}", e),
    }
}

pub fn create_other_files() {
    match utils::create_file_nn(Path::new(consts::filepaths::BANNED_IP)) {
        Ok(_) => info!("Created file {}", consts::filepaths::BANNED_IP),
        Err(e) => info!(
            "Failed to create the file {} as error:{}",
            consts::filepaths::BANNED_IP,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::filepaths::BANNED_PLAYERS)) {
        Ok(_) => info!("Created file {}", consts::filepaths::BANNED_PLAYERS),
        Err(e) => info!(
            "Failed to create the file {} as error:{}",
            consts::filepaths::BANNED_PLAYERS,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::filepaths::OPERATORS)) {
        Ok(_) => info!("Created file {}", consts::filepaths::OPERATORS),
        Err(e) => info!(
            "Failed to create the file {} as error:{}",
            consts::filepaths::OPERATORS,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::filepaths::SESSION)) {
        Ok(_) => info!("Created file {}", consts::filepaths::SESSION),
        Err(e) => info!(
            "Failed to create the file {} as error:{}",
            consts::filepaths::SESSION,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::filepaths::USERCACHE)) {
        Ok(_) => info!("Created file {}", consts::filepaths::USERCACHE),
        Err(e) => info!(
            "Failed to create the file {} as error:{}",
            consts::filepaths::USERCACHE,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::filepaths::WHITELIST)) {
        Ok(_) => info!("Created file {}", consts::filepaths::WHITELIST),
        Err(e) => info!(
            "Failed to create the file {} as error:{}",
            consts::filepaths::WHITELIST,
            e
        ),
    }
}
pub fn create_dirs() {
    match utils::create_dir(Path::new(consts::folderpath::LOGS)) {
        Ok(_) => info!("Created dir{}", consts::folderpath::LOGS),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::folderpath::LOGS,
            e
        ),
    }

    match utils::create_dir(Path::new(consts::folderpath::WORLDS_DIRECTORY)) {
        Ok(_) => info!("No existing world data, creating new world"),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::folderpath::WORLDS_DIRECTORY,
            e
        ),
    }

    match utils::create_dir(Path::new(consts::folderpath::OVERWORLD)) {
        Ok(_) => info!("Created dir{}", consts::folderpath::OVERWORLD),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::folderpath::OVERWORLD,
            e
        ),
    }

    match utils::create_dir(Path::new(consts::folderpath::THE_END)) {
        Ok(_) => info!("Created dir{}", consts::folderpath::THE_END),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::folderpath::THE_END,
            e
        ),
    }

    match utils::create_dir(Path::new(consts::folderpath::NETHER)) {
        Ok(_) => info!("Created dir{}", consts::folderpath::NETHER),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::folderpath::NETHER,
            e
        ),
    }
}
