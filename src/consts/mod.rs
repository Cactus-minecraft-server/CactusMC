//! This module is where we store constants, like filepaths or the the version of the current
//! Minecraft version that the server is implementing.
// TODO: Maybe reimplement this with a real querying API, like a HashMap like object.

/// Module where we store information relevant to the Minecraft server.
pub mod minecraft {
    pub const VERSION: &str = "1.21.4";
    pub const PROTOCOL_VERSION: usize = 769;
}

/// Server logging messages.
pub mod messages {

    use colored::*;
    use once_cell::sync::Lazy;

    use super::minecraft::VERSION;

    pub static SERVER_STARTING: Lazy<String> = Lazy::new(|| {
        format!("Starting minecraft server version {}", VERSION)
            .bold()
            .to_string()
    });

    pub static SERVER_STARTED: Lazy<String> =
        Lazy::new(|| "[ SERVER STARTED ]".bright_green().bold().to_string());

    pub static SERVER_SHUTDOWN_SUCCESS: Lazy<String> =
        Lazy::new(|| "[ SERVER SHUT DOWN ]".bright_red().bold().to_string());

    pub static SERVER_SHUTDOWN_ERROR: Lazy<String> = Lazy::new(|| {
        "[ SERVER SHUT DOWN WITH ERROR ]"
            .bright_red()
            .bold()
            .to_string()
    });

    pub static SERVER_SHUTDOWN_CTRL_C: Lazy<String> = Lazy::new(|| {
        "[ SERVER SHUT DOWN WITH CTRL+C ]"
            .bright_red()
            .bold()
            .to_string()
    });

    pub static GREET: Lazy<String> =
        Lazy::new(|| "Hello, world from Cactus!".green().bold().to_string());

    /// Used when exiting the server with an exit code.
    pub fn server_shutdown_code(code: i32) -> String {
        format!("[ server shutdown with code: {code}]")
            .to_uppercase()
            .bright_red()
            .bold()
            .to_string()
    }
}

/// Module used to store file paths relative to the server binary.
pub mod file_paths {
    /// server.properties file, used to store server settings.
    pub const PROPERTIES: &str = "server.properties";
    pub const EULA: &str = "eula.txt";
    pub const OPERATORS: &str = "ops.json";
    pub const WHITELIST: &str = "whitelist.json";
    pub const BANNED_IP: &str = "banned-ips.json";
    pub const BANNED_PLAYERS: &str = "banned-players.json";
    pub const USERCACHE: &str = "usercache.json";
    pub const SESSION: &str = "session.lock";
    pub const SERVER_ICON: &str = "server-icon.png";
}

pub mod directory_paths {
    pub const WORLDS_DIRECTORY: &str = "world/";
    pub const THE_END: &str = "world/DIM1/";
    pub const NETHER: &str = "world/DIM-1/";
    pub const OVERWORLD: &str = "world/region/";
    pub const LOGS: &str = "logs/";
}

pub mod file_contents {
    use crate::time;

    /// Returns the default content of the 'eula.txt' file.
    pub fn eula() -> String {
        let mut content = String::new();

        content += "# By changing the setting below to 'true' you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).\n";
        content += &format!("# {}", time::get_formatted_time());
        content += "\neula=false";
        content
    }

    /// Returns the default content of the 'server.properties' file.
    pub fn server_properties() -> String {
        const SERVER_PROPERTIES_INNER: &str = r#"accepts-transfers=false
allow-flight=false
allow-nether=true
broadcast-console-to-ops=true
broadcast-rcon-to-ops=true
bug-report-link=
difficulty=normal
enable-command-block=false
enable-jmx-monitoring=false
enable-query=false
enable-rcon=false
enable-status=true
enforce-secure-profile=true
enforce-whitelist=false
entity-broadcast-range-percentage=100
force-gamemode=false
function-permission-level=2
gamemode=survival
generate-structures=true
generator-settings={}
hardcore=false
hide-online-players=false
initial-disabled-packs=
initial-enabled-packs=vanilla
level-name=world
level-seed=
level-type=minecraft\:normal
log-ips=true
max-chained-neighbor-updates=1000000
max-players=20
max-tick-time=60000
max-world-size=29999984
motd=A beautiful CactusMC server!
network-compression-threshold=256
online-mode=true
op-permission-level=4
player-idle-timeout=0
prevent-proxy-connections=false
pvp=true
query.port=25565
rate-limit=0
rcon.password=
rcon.port=25575
region-file-compression=deflate
require-resource-pack=false
resource-pack=
resource-pack-id=
resource-pack-prompt=
resource-pack-sha1=
server-ip=
server-port=25565
simulation-distance=10
spawn-animals=true
spawn-monsters=true
spawn-npcs=true
spawn-protection=16
sync-chunk-writes=true
text-filtering-config=
use-native-transport=true
view-distance=10
white-list=false"#;

        format!(
            "# Minecraft server properties\n# {}\n{}",
            time::get_formatted_time(),
            SERVER_PROPERTIES_INNER
        )
    }
}

/// Strings for packets
pub mod protocol {

    use base64::{engine::general_purpose, Engine};
    use image::{GenericImageView, ImageFormat};
    use log::error;
    use serde_json::json;

    use crate::{config::Settings, gracefully_exit};

    use super::file_paths::SERVER_ICON;

    /// Returns the Base64-encoded server icon.
    /// The image must be a 64x64 PNG image as the file server-icon.png
    fn get_favicon() -> Result<String, Box<dyn std::error::Error>> {
        let file_data = std::fs::read(SERVER_ICON)?;

        // Guess the image format
        let format = image::guess_format(&file_data)?;
        if format != ImageFormat::Png {
            return Err("The server icon must be in PNG format".into());
        }

        // Load the image to verify dimensions
        let img = image::load_from_memory_with_format(&file_data, format)?;
        if img.dimensions() != (64, 64) {
            return Err("The server icon must have dimensions of 64x64".into());
        }

        // Encode to Base64
        let base64_icon = general_purpose::STANDARD.encode(file_data);

        // Construct the Data URI
        let favicon = format!("data:image/png;base64,{base64_icon}");
        Ok(favicon)
    }

    /// Returns the Status Response JSON.
    pub fn status_response_json() -> String {
        let config = Settings::new();

        let version_name = super::minecraft::VERSION;
        let protocol = super::minecraft::PROTOCOL_VERSION;
        let max_players = config.max_players;

        // TODO: This does not mirror the server's current state.
        let online_players = 0;

        let description_text = config.motd;

        // TODO: Implement logic such that, if no icon is provided, not include it in the JSON.
        if let Err(err) = get_favicon() {
            error!("Server icon not found: {err}. Shutting down the server...");
            gracefully_exit(crate::ExitCode::Failure);
        }
        let favicon = get_favicon().unwrap();

        let enforces_secure_chat = config.enforce_secure_profile;

        let json_data = json!({
            "version": {
                "name": version_name,
                "protocol": protocol
            },
            "players": {
                "max": max_players,
                "online": online_players,
            },
            "description": {
                "text": description_text
            },
            "favicon": favicon,
            "enforcesSecureChat": enforces_secure_chat
        });

        serde_json::to_string(&json_data).unwrap()
    }
}
