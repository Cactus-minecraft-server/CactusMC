//! This module is where we store constants, like filepaths or the the version of the current
//! Minecraft version that the server is implementing.
// TODO: Maybe reimplement this with a real querying API, like a HashMap like object.

/// Module where we store information relevant to the Minecraft server.
pub mod minecraft {
    pub const VERSION: &'static str = "1.21.1"; //upgrade to 1.21.1 cuz wiki.vg is up to date
    pub const PROTOCOL_VERSION: usize = 767;
}

/// Server logging messages.
pub mod messages {

    use colored::*;
    use once_cell::sync::Lazy;

    pub static SERVER_STARTING: Lazy<String> =
        Lazy::new(|| "[ SERVER STARTING... ]".bold().to_string());

    pub static SERVER_STARTED: Lazy<String> =
        Lazy::new(|| "[ SERVER STARTED ]".bright_green().bold().to_string());

    pub static SERVER_SHUTDOWN: Lazy<String> =
        Lazy::new(|| "[ SERVER SHUT DOWN ]".bright_red().bold().to_string());

    pub static GREET: Lazy<String> =
        Lazy::new(|| "Hello, world from Copper!".green().bold().to_string());

    /// Used when exiting the server with an exit code.
    pub fn server_shutdown_code(code: i32) -> String {
        return format!("[ SERVER SHUT DOWN WITH CODE: {code}]")
            .bright_red()
            .bold()
            .to_string();
    }
}

/// Module used to store file paths relative to the server binary.
pub mod filepaths {
    /// server.properties file, used to store server settings.
    pub const PROPERTIES: &'static str = "server.properties";
    pub const EULA: &'static str = "eula.txt";
}

pub mod file_content {
    use crate::time;

    /// Returns the default content of the 'eula.txt' file.
    pub fn eula() -> String {
        let mut content = String::new();
        let formatted_time: String = format!("# {}",time::get_formatted_time());

        content += "# By changing the setting below to 'true' you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).\n";
        content += &formatted_time;
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
difficulty=easy
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
motd=A Minecraft Server
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
            "# Minecraft server properties\n{}\n{}",
            format!("# {}",time::get_formatted_time()),
            SERVER_PROPERTIES_INNER,
        )
    }
}
