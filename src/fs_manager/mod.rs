use std::fs::{self, remove_dir_all, File, OpenOptions};
use std::io::{self, BufRead, Seek, SeekFrom};
use std::path::Path;
mod utils;
use crate::config::{Difficulty, Gamemode};
use crate::{config, consts, gracefully_exit};
use cactus_world::level::{create_nbt, LevelDat, VersionInfo};
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::io::Write;

// Initializes the server's required files and directories
pub fn init() -> std::io::Result<()> {
    eula()?;
    create_server_properties()
}

/// Checks if the eula is agreed, if not shutdown the server with failure code.
fn eula() -> io::Result<()> {
    let path = Path::new(consts::file_paths::EULA);
    if !path.exists() {
        create_eula()?;
        let content = "Please agree to the 'eula.txt' and start the server again.";
        warn!("{}", content.bright_red().bold());
        gracefully_exit(crate::ExitCode::Failure);
    } else {
        let is_agreed_eula = check_eula()?;
        if !is_agreed_eula {
            let error_content = "Cannot start the server, please agree to the 'eula.txt'";
            error!("{}", error_content.bright_red().bold().blink());
            gracefully_exit(crate::ExitCode::Failure);
        }
        Ok(())
    }
}

/// Creates the 'server.properties' file if it does not already exist.
fn create_server_properties() -> io::Result<()> {
    let path = Path::new(consts::file_paths::PROPERTIES);
    let content = consts::file_contents::server_properties();

    utils::create_file(path, &content)
}

/// Creates the 'eula.txt' file if it does not already exist.
fn create_eula() -> io::Result<()> {
    let path = Path::new(consts::file_paths::EULA);
    let content = consts::file_contents::eula();

    utils::create_file(path, &content)
}

/// Check if the 'eula.txt' has been agreed to.
fn check_eula() -> io::Result<bool> {
    let file = File::open(Path::new(consts::file_paths::EULA))?;
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
    match utils::create_file_nn(Path::new(consts::file_paths::BANNED_IP)) {
        Ok(_) => (),
        Err(e) => error!(
            "Failed to create the file {} as error:{}",
            consts::file_paths::BANNED_IP,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::file_paths::BANNED_PLAYERS)) {
        Ok(_) => (),
        Err(e) => error!(
            "Failed to create the file {} as error:{}",
            consts::file_paths::BANNED_PLAYERS,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::file_paths::OPERATORS)) {
        Ok(_) => (),
        Err(e) => error!(
            "Failed to create the file {} as error:{}",
            consts::file_paths::OPERATORS,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::file_paths::SESSION)) {
        Ok(_) => (),
        Err(e) => error!(
            "Failed to create the file {} as error:{}",
            consts::file_paths::SESSION,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::file_paths::USERCACHE)) {
        Ok(_) => (),
        Err(e) => error!(
            "Failed to create the file {} as error:{}",
            consts::file_paths::USERCACHE,
            e
        ),
    }
    match utils::create_file_nn(Path::new(consts::file_paths::WHITELIST)) {
        Ok(_) => (),
        Err(e) => error!(
            "Failed to create the file {} as error:{}",
            consts::file_paths::WHITELIST,
            e
        ),
    }
    match create_level_file() {
        Ok(_) => (),
        Err(e) => error!(
            "Failed to create the file at {} as error {}",
            consts::file_paths::LEVEL,
            e
        ),
    }
}
pub fn create_dirs() {
    match utils::create_dir(Path::new(consts::directory_paths::LOGS)) {
        Ok(_) => (),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::directory_paths::LOGS,
            e
        ),
    }

    match utils::create_dir(Path::new(consts::directory_paths::WORLDS_DIRECTORY)) {
        Ok(_) => (),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::directory_paths::WORLDS_DIRECTORY,
            e
        ),
    }

    match utils::create_dir(Path::new(consts::directory_paths::OVERWORLD)) {
        Ok(_) => (),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::directory_paths::OVERWORLD,
            e
        ),
    }

    match utils::create_dir(Path::new(consts::directory_paths::THE_END)) {
        Ok(_) => (),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::directory_paths::THE_END,
            e
        ),
    }

    match utils::create_dir(Path::new(consts::directory_paths::NETHER)) {
        Ok(_) => (),
        Err(e) => info!(
            "Failed to create dir{} as error: {}",
            consts::directory_paths::NETHER,
            e
        ),
    }
}
#[derive(Serialize, Deserialize)]
struct Player {
    uuid: String,
    name: String,
    level: u8,
    bypassesPlayerLimit: bool,
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
        .create(true)
        .open(filename)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    if content.starts_with('\u{feff}') {
        content = content.trim_start_matches('\u{feff}').to_string();
    }

    let mut players: Vec<Player> = if content.trim().is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
    };

    if !players.iter().any(|p| p.uuid == uuid) {
        players.push(Player {
            uuid: uuid.to_string(),
            name: name.to_string(),
            level,
            bypassesPlayerLimit: bypasses_player_limit,
        });
        info!("Made {} a server operator", name.to_string())
    } else {
        warn!("Nothing changed. The player already is an operator")
    }

    // Réécrire le fichier avec le contenu mis à jour
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(serde_json::to_string_pretty(&players)?.as_bytes())?;

    Ok(())
}
/// Removes all files related to the server, excluding the server.
///
/// I am not sure if this is a good idea, because it takes some time to maintain and is not very
/// useful but it's mostly for dev purpose.
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
        consts::file_paths::EULA,
        consts::file_paths::PROPERTIES,
        consts::file_paths::BANNED_IP,
        consts::file_paths::BANNED_PLAYERS,
        consts::file_paths::OPERATORS,
        consts::file_paths::SESSION,
        consts::file_paths::USERCACHE,
        consts::file_paths::WHITELIST,
        consts::file_paths::LEVEL,
    ];

    // Delete files using the `remove_file` helper function
    for file in &files {
        remove_file(file)?;
    }

    // List all directories to be deleted
    let directories = [
        consts::directory_paths::LOGS,
        consts::directory_paths::NETHER,
        consts::directory_paths::OVERWORLD,
        consts::directory_paths::THE_END,
        consts::directory_paths::WORLDS_DIRECTORY,
    ];

    // Delete directories using the `remove_dir` helper function
    for dir in &directories {
        remove_dir_all(dir)?;
    }
    let info = "[Info]".green();
    println!(
        "{} Files cleaned successfully before starting the server.",
        info
    );
    gracefully_exit(crate::ExitCode::Success);
}
fn create_level_file() -> Result<(), std::io::Error> {
    if Path::new(consts::file_paths::LEVEL).exists() {
        info!("level.dat file already exist, not altering it");
        return Ok(());
    }
    let server_version = VersionInfo {
        id: 4440,
        snapshot: false,
        series: "main".into(),
        name: consts::minecraft::VERSION.into(),
    };
    let data = LevelDat {
        version: server_version,
        difficulty: match config::Settings::new().difficulty {
            Difficulty::Easy => 0,
            Difficulty::Normal => 1,
            Difficulty::Hard => 2,
        },
        game_type: match config::Settings::new().gamemode {
            Gamemode::Survival => 0,
            Gamemode::Creative => 1,
            Gamemode::Spectator => 3,
            Gamemode::Adventure => 2,
        },

        hardcore: config::Settings::new().hardcore,
        level_name: match config::Settings::new().level_name {
            Some(words) => words,
            _ => "Overworld".into(),
        },

        ..Default::default()
    };

    let result = create_nbt(&data, consts::file_paths::LEVEL);
    if result.is_ok() {
        info!("File created at {}", consts::file_paths::LEVEL)
    }
    result
}
