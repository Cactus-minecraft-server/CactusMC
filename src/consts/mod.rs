//! This module is where we store constants, like filepaths or the the version of the current
//! Minecraft version that the server is implementing.

// TODO: Maybe reimplement this with a real querying API, like a HashMap like object.

/// Module where we store information relevant to the Minecraft server.
pub mod minecraft {
    pub const VERSION: &'static str = "1.21"; // I just change the game version to 1.20.6 -> 1.21
    pub const PROTOCOL_VERSION: usize = 767;
}

/// Module where we store constant file paths used by the server.
pub mod filepaths {
    /// server.properties file, used to store server settings.
    pub const PROPERTIES: &'static str = "server.properties"; //let's keep that, if it's not working put a .toml
}
