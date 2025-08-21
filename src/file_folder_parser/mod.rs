use std::fs::File;
use std::io::{self, Write};
// Importer std::io pour utiliser io::Result et io::Error

pub fn create_server_properties(content: &str) -> io::Result<()> {
    // create the file
    let mut file = File::create("server_test.properties")?;
    //write into the file
    file.write_all(content.as_bytes())?; // chatgpt says me rust want have bytes for write anything ...

    Ok(())
}
