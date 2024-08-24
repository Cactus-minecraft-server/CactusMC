use std::fs::File;
use std::io::{self, Write ,BufRead};
use std::path::Path;


use chrono::{DateTime, Local, Utc};
use colored::Colorize;


pub fn create_server_properties(content:&str,file_path:&'static str) -> io::Result<()> {
    let path = Path::new(file_path);

    // verify if the file already exist
    if path.exists() {
        println!("the file \"{}\" already exist, the programm will use this one.",file_path.blue());
    } else {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        println!("the file \"{}\" have been created.",file_path.blue())
    }
    Ok(())
}//test passed

pub fn create_eula(file_path:&'static str) -> io::Result<()> {
    let now = Utc::now();
    let local_time: DateTime<Local> = now.with_timezone(&Local); //convert to local machine time
    let formatted_time = local_time.format("%a %b %d %H:%M:%S %Y").to_string(); //format the time


    let final_input = format!(
        "#By changing the setting below to TRUE you are indicating your agreement to our EULA (https://account.mojang.com/documents/minecraft_eula).\n#{}\neula=false",
        formatted_time,

    );

    let path = Path::new(file_path);
    if path.exists() {
        println!("the file {} is already created",file_path.green());
    } else {
        let mut file = File::create(path)?;
        file.write_all(final_input.as_bytes())?;
        println!("Creation of the file {}",file_path.red())
    }

    Ok(())
}


pub fn check_eula(path: &'static str) -> bool {
    println!("Reading the file {}…",path);
    if let Ok(file) = File::open(Path::new(path)) {
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("eula=") {
                    let eula_value = line.split('=').nth(1).unwrap_or("");
                    return eula_value == "true";
                }
            }
        }
    }

    false
}



