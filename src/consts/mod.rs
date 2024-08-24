//! This module is where we store constants, like filepaths or the the version of the current
//! Minecraft version that the server is implementing.

// TODO: Maybe reimplement this with a real querying API, like a HashMap like object.

/// Module where we store information relevant to the Minecraft server.
pub mod minecraft {
    pub const VERSION: &'static str = "1.21.1"; //upgrade to 1.21.1 cuz wiki.vg is up to date
    pub const PROTOCOL_VERSION: usize = 767;
}

/// Module used to store file paths relative to the server binary.
pub mod filepaths {
    /// server.properties file, used to store server settings.
    pub const PROPERTIES: &'static str = "server.properties";
    pub const EULA: &'static str = "eula.txt";
}

pub mod file_content {
    pub const SERVER_PROPERTIES: &str = r#"accepts-transfers=false
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

    pub const EULA: &str = r"eula=false";
}
