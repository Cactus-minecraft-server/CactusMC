use std::{fs::File, io::{self, Write}, path::Path};
use colored::Colorize;
pub fn check_dir(dir_path:&Path) -> bool {
    if dir_path.exists() {
        println!("The folder \"{}\" already exists, the program will use this one.",dir_path.to_string_lossy().red() );

        true
    }else{
        println!("The folder \"{}\" already exists, the program will use this one.",dir_path.to_string_lossy().red() );

        false
    }
}

pub fn check_file(file_path:&Path) -> bool {
    if file_path.exists() {
        println!("The file \"{}\" already exists, the program will use this one.",file_path.to_string_lossy().red() );
        true
    }else{
        println!("The file \"{}\" has been created.",file_path.to_string_lossy().blue());
        false
    }
}

pub fn create_file(path:&Path,content:&str) ->Result<(),io::Error>{
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    println!("The file \"{}\" has been created.",path.to_string_lossy().blue());
    Ok(())
}
