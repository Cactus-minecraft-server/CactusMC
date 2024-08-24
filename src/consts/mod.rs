//! This module is where we store constants, like filepaths or the the version of the current
//! Minecraft version that the server is implementing.

// TODO: Maybe reimplement this with a real querying API, like a HashMap like object.

/// Module where we store information relevant to the Minecraft server.
pub mod minecraft {
    pub const VERSION: &'static str = "1.21.1"; //upgrade to 1.21.1 cuz wiki.vg is up to date
    pub const PROTOCOL_VERSION: usize = 767;
}

/// Module where we store constant file paths used by the server.
pub mod filepaths {
    /// server.properties file, used to store server settings.
    pub const PROPERTIES: &'static str = "server.properties"; //let's keep that, if it's not working put a .toml
    pub const EULA: &'static str = "eula.txt";
}
pub mod file_content{
    pub const SERVER_PROPERTIES:&str = r#"enable-jmx-monitoring=false
rcon.port=25575
level-seed=
gamemode=survival
enable-command-block=false
enable-query=false
generator-settings={}
enforce-secure-profile=true
level-name=world
motd=A Minecraft Server
query.port=25565
pvp=true
generate-structures=true
max-chained-neighbor-updates=1000000
difficulty=easy
network-compression-threshold=256
max-tick-time=60000
require-resource-pack=false
use-native-transport=true
max-players=20
online-mode=true
enable-status=true
allow-flight=false
initial-disabled-packs=
broadcast-rcon-to-ops=true
view-distance=10
server-ip=
resource-pack-prompt=
allow-nether=true
server-port=25565
enable-rcon=false
sync-chunk-writes=true
op-permission-level=4
prevent-proxy-connections=false
hide-online-players=false
resource-pack=
entity-broadcast-range-percentage=100
simulation-distance=10
rcon.password=
player-idle-timeout=0
force-gamemode=false
rate-limit=0
hardcore=false
white-list=false
broadcast-console-to-ops=true
spawn-npcs=true
spawn-animals=true
log-ips=true
function-permission-level=2
initial-enabled-packs=vanilla
level-type=minecraft\:normal
text-filtering-config=
spawn-monsters=true
enforce-whitelist=false
spawn-protection=16
resource-pack-sha1=
max-world-size=29999984"#;





}

