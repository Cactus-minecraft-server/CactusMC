use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead};
use std::path::Path;
use std::vec;
mod utils;
use crate::{consts, gracefully_exit};
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Read;
use std::io::Write;

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

    utils::create_file(path, &content)
}

/// Creates the 'eula.txt' file if it does not already exist.
fn create_eula() -> io::Result<()> {
    let path = Path::new(consts::filepaths::EULA);
    let content = consts::file_content::eula();

    utils::create_file(path, &content)
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
#[derive(Serialize, Deserialize)]
struct Player {
    uuid: String,
    name: String,
    level: u8,
    bypassesplayerlimit: bool,
}

pub fn write_ops_json(
    filename: &str,
    uuid: &str,
    name: &str,
    level: u8,
    bypasses_player_limit: bool,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(consts::filepaths::OPERATORS)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut json_data: Vec<Value> = if content.trim().is_empty() {
        vec![]
    } else {
        serde_json::from_str(&content)?
    };
    let new_object = json!({
        "name": name,
        "uuid": uuid,
        "level": level,
        "bypassesPlayerLimit": bypasses_player_limit
    });
    json_data.push(new_object);
    file.set_len(0)?;
    if let Err(e) = file.write_all(serde_json::to_string_pretty(&json_data)?.as_bytes()) {
        warn!("Failed to write to ops: {e}");
    }
    Ok(())
}

/// Removes all files related to the server, excluding the server.
///
/// I am not sure if this is a good idea, because it takes some time to maintain and is not very
/// useful.
pub fn clean_files() -> Result<(), std::io::Error> {
    // Define a helper function to handle file removals
    fn remove_file(file_path: &str) -> Result<(), std::io::Error> {
        match fs::remove_file(file_path) {
            Ok(_) => {
                info!("File deleted: {}", file_path);
                Ok(())
            }
            Err(e) => {
                info!("Error when deleting file {}: {}", file_path, e);
                Err(e)
            }
        }
    }

    // Define a helper function to handle directory removals
    fn remove_dir(dir_path: &str) -> Result<(), std::io::Error> {
        match fs::remove_dir(dir_path) {
            Ok(_) => {
                info!("Directory deleted: {}", dir_path);
                Ok(())
            }
            Err(e) => {
                info!("Error when deleting directory {}: {}", dir_path, e);
                Err(e)
            }
        }
    }

    // List all files to be deleted
    let files = [
        consts::filepaths::EULA,
        consts::filepaths::PROPERTIES,
        consts::filepaths::BANNED_IP,
        consts::filepaths::BANNED_PLAYERS,
        consts::filepaths::OPERATORS,
        consts::filepaths::SESSION,
        consts::filepaths::USERCACHE,
        consts::filepaths::WHITELIST,
    ];

    // Delete files using the `remove_file` helper function
    for file in &files {
        remove_file(file)?;
    }

    // List all directories to be deleted
    let directories = [
        consts::folderpath::LOGS,
        consts::folderpath::NETHER,
        consts::folderpath::OVERWORLD,
        consts::folderpath::THE_END,
        consts::folderpath::WORLDS_DIRECTORY,
    ];

    // Delete directories using the `remove_dir` helper function
    for dir in &directories {
        remove_dir(dir)?;
    }

    info!("Files cleaned successfully before starting the server.");
    gracefully_exit(0);
}
