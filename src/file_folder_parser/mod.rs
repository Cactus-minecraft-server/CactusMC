use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use colored::Colorize;

use crate::config;


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
    if path.exists() && check_eula(file_path) == true {
        println!("You already agreed too {}",file_path.green());
    } else {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        println!("You have to agree to {} before run the server",file_path.red())
    }
    Ok(())
}
pub fn check_eula(file_path:&'static str) ->bool{
    let path = Path::new(file_path);
    let eula_result = config::read(path).expect("error reading eula_file");
    let agree = eula_result.get_property("eula").unwrap().parse::<bool>().unwrap();
    agree


}
