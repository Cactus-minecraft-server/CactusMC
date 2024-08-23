use std::fmt::Error;

pub fn create_server_properties() ->Result<(),Error> {
    let mut file = File::create("server.properties")?;
    file.write_all(b"test")?;
    Ok(())
}
