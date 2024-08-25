use colored::Colorize;
use log::{error, info};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
mod utils;
use crate::consts;
use crate::gracefully_exit;
use chrono::{DateTime, Local, Utc};

// Initializes the server's required files and directories
pub fn init() -> std::io::Result<()> {
    // Get time
    let now = Utc::now();
    let local_time: DateTime<Local> = now.with_timezone(&Local); // Convert to local machine time
    let formatted_time = local_time.format("%a %b %d %H:%M:%S %Y").to_string(); // Format the time

    // eula.txt
    let _ = create_eula(consts::filepaths::EULA, &formatted_time)?;

    if !check_eula(consts::filepaths::EULA) {
        const ERROR_MESSAGE: &str = "Cannot start the server. Please agree to the 'eula.txt'.";
        error!("{ERROR_MESSAGE}");
        gracefully_exit(-1);
    }

    // server.properties
    let _ = create_server_properties(
        consts::file_content::SERVER_PROPERTIES,
        consts::filepaths::PROPERTIES,
        &formatted_time,
    )?;

    Ok(())
}

fn create_server_properties(
    content: &str,
    file_path: &'static str,
    formatted_time: &str,
) -> io::Result<()> {
    let path = Path::new(file_path);
    let final_input = format!(
        "#Minecraft server properties\n#{}\n{}",
        formatted_time, content,
    );
    // verify if the file already exist
    if utils::check_file(path) == false {
        let mut file = File::create(path)?;
        file.write_all(final_input.as_bytes())?;
        info!("The file \"{}\" has been created.", file_path.blue())
    }
    Ok(())
} //test passed

fn create_eula(file_path: &'static str, formatted_time: &str) -> io::Result<()> {
    let final_input = format!(
        "#By changing the setting below to TRUE you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).\n#{}\neula=false",
        formatted_time,

    );

    let path = Path::new(file_path);
    if utils::check_file(path) == false {
        let mut file = File::create(path)?;
        file.write_all(final_input.as_bytes())?;
        info!("Creation of the file {}", file_path.red())
    }

    Ok(())
}

fn check_eula(path: &'static str) -> bool {
    info!("Reading the file {}…", path);
    if let Ok(file) = File::open(Path::new(path)) {
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("eula=") {
                    let eula_value = line.split('=').nth(1).unwrap_or("").to_lowercase();
                    return eula_value == "true";
                }
            }
        }
    }

    false
}
