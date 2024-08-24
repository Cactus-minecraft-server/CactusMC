use std::fs::File;
use std::io::{self, Write ,BufRead};
use std::path::Path;

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

pub fn create_eula(content:&str,file_path:&'static str) -> io::Result<()> {
    let path = Path::new(file_path);
    if path.exists() {
        println!("the file {} is already created",file_path.green());
    } else {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
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



