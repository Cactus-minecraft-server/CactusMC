use std::fs::File;
use std::io::{self, Write};
use std::path::Path;


pub fn create_server_properties(content:&str,file_name:&'static str) -> io::Result<()> {
    let path = Path::new(file_name);

    // create the file
    if path.exists() {
        println!("the file \"{}\" already exist, the programm will use this one.",file_name);
    } else {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        println!("the file \"{}\" have been created.",file_name)
    }
    Ok(())
}//to test
