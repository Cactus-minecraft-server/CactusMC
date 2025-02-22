//! This module is the interface between the server.properties file. Querying for server settings.
use core::fmt;
// !TODO generator_settings
// !Todo text-filtering-config
// use dot_properties::{read_properties, Properties};
use read_properties::Properties;
use std::fs::File;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::net::Ipv4Addr;
use std::path::Path;
use std::str::FromStr;
pub mod read_properties;
//use std::sync::Arc;

/// Function to get a `Properties` object to which the caller can then query keys.
///
/// # Example
/// ```rust
/// let config_file = config::read(Path::new(consts::filepaths::PROPERTIES))
///     .expect("Error reading server.properties file");
///
/// let difficulty = config_file.get_property("difficulty").unwrap();
/// let max_players = config_file.get_property("max_players").unwrap();
///
/// // Note that `get_property()` returns a `Result<&'_ str, PropertyNotFoundError<'a>>`
/// // So everything's a string slice.
/// println!("{difficulty}");
/// println!("{max_players}");
/// ```
///
#[derive(Debug)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Debug)]
pub enum Gamemode {
    Adventure,
    Survival,
    Creative,
    Spectator,
}

#[derive(Debug)]
pub enum WorldPreset {
    Normal,
    Flat,
    LargeBiomes,
    Amplified,
    SingleBiomeSurface,
}

// TODO: Maybe make Settings a singleton

#[derive(Debug)]
pub struct Settings {
    pub enable_jmx_monitoring: bool,
    pub rcon_port: u16,
    pub level_seed: Option<i64>,
    pub gamemode: Gamemode,
    pub enable_command_block: bool,
    pub enable_query: bool,
    pub enforce_secure_profile: bool,
    pub level_name: Option<String>,
    pub motd: Option<String>,
    pub query_port: u16,
    pub pvp: bool,
    pub generate_structures: bool,
    pub max_chained_neighbor_updates: Option<i32>,
    pub difficulty: Difficulty,
    pub network_compression_threshold: i32,
    pub max_tick_time: i64,
    pub require_resource_pack: bool,
    pub use_native_transport: bool,
    pub max_players: u32,
    pub online_mode: bool,
    pub enable_status: bool,
    pub allow_flight: bool,
    pub initial_disabled_packs: Option<String>,
    pub broadcast_rcon_to_ops: bool,
    pub view_distance: u8,
    pub server_ip: Option<Ipv4Addr>,
    pub resource_pack_prompt: Option<String>,
    pub allow_nether: bool,
    pub server_port: u16,
    pub enable_rcon: bool,
    pub sync_chunk_writes: bool,
    pub op_permission_level: u8,
    pub prevent_proxy_connections: bool,
    pub hide_online_players: bool,
    pub resource_pack: Option<String>,
    pub entity_broadcast_range_percentage: u8,
    pub simulation_distance: u8,
    pub rcon_password: Option<String>,
    pub player_idle_timeout: i32,
    pub force_gamemode: bool,
    pub rate_limit: u32,
    pub hardcore: bool,
    pub white_list: bool,
    pub broadcast_console_to_ops: bool,
    pub spawn_npcs: bool,
    pub spawn_animals: bool,
    pub log_ips: bool,
    pub function_permission_level: u8,
    pub initial_enabled_packs: String,
    pub level_type: WorldPreset,
    pub spawn_monsters: bool,
    pub enforce_whitelist: bool,
    pub spawn_protection: u16,
    pub resource_pack_sha1: Option<String>,
    pub max_world_size: u32,
    //generator_settings:todo!(),
    //text_filtering_config:todo!(),
}

fn read(filepath: &Path) -> std::io::Result<Properties> {
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    read_properties::read_properties(&mut reader)
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}
impl FromStr for Gamemode {
    type Err = ();

    fn from_str(input: &str) -> Result<Gamemode, Self::Err> {
        match input.to_lowercase().as_str() {
            "creative" => Ok(Gamemode::Creative),
            "survival" => Ok(Gamemode::Survival),
            "spectator" => Ok(Gamemode::Spectator),
            "adventure" => Ok(Gamemode::Adventure),
            _ => Err(()),
        }
    }
}
impl FromStr for WorldPreset {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "normal" => Ok(WorldPreset::Normal),
            "flat" => Ok(WorldPreset::Flat),
            "largebiomes" => Ok(WorldPreset::LargeBiomes),
            "amplified" => Ok(WorldPreset::Amplified),
            "singlebiomesurface" => Ok(WorldPreset::SingleBiomeSurface),
            _ => Err(()),
        }
    }
}
impl FromStr for Difficulty {
    type Err = ();

    fn from_str(input: &str) -> Result<Difficulty, Self::Err> {
        match input.to_lowercase().as_str() {
            "easy" => Ok(Difficulty::Easy),
            "normal" => Ok(Difficulty::Normal),
            "hard" => Ok(Difficulty::Hard),
            _ => Err(()), // Gère les valeurs inconnues
        }
    }
}
impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let difficulty = match self {
            Difficulty::Easy => "easy",
            Difficulty::Normal => "normal",
            Difficulty::Hard => "hard",
        };
        write!(f, "{}", difficulty)
    }
}

impl Settings {
    pub fn new() -> Self {
        let config_file = read(Path::new(crate::consts::file_paths::PROPERTIES))
            .expect("Error reading {server.properties} file");
        Self {
            enable_jmx_monitoring: Self::get_bool(&config_file, "enable-jmx-monitoring", false),
            rcon_port: Self::get_u16(&config_file, "rcon.port", 25575),
            level_seed: Self::get_optional_i64(&config_file, "level-seed"),
            gamemode: Self::get_enum::<Gamemode>(&config_file, "gamemode", Gamemode::Survival),
            enable_command_block: Self::get_bool(&config_file, "enable-command-block", false),
            max_players: Self::get_u32(&config_file, "max-players", 20),
            server_ip: Self::get_optional_ip(&config_file, "server-ip"),
            server_port: Self::get_u16(&config_file, "server-port", 25565),
            allow_flight: Self::get_bool(&config_file, "allow-flight", false),
            allow_nether: Self::get_bool(&config_file, "allow-nether", true),
            broadcast_console_to_ops: Self::get_bool(
                &config_file,
                "broadcast-console-to-ops",
                true,
            ),
            difficulty: config_file
                .get_property("difficulty")
                .unwrap_or("easy")
                .parse::<Difficulty>()
                .unwrap_or(Difficulty::Easy),
            broadcast_rcon_to_ops: Self::get_bool(&config_file, "broadcast-rcon-to-ops", true),
            enable_query: Self::get_bool(&config_file, "enable-query", true),
            enable_rcon: Self::get_bool(&config_file, "enable-rcon", false),
            enable_status: Self::get_bool(&config_file, "enable-status", true),
            enforce_secure_profile: Self::get_bool(&config_file, "enforce-secure-profil", true),
            enforce_whitelist: Self::get_bool(&config_file, "enforce-whitelist", false),
            entity_broadcast_range_percentage: Self::get_u8(
                &config_file,
                "entity-broadcast-range-percentage",
                100,
            ),
            force_gamemode: Self::get_bool(&config_file, "force-gamemode", false),
            function_permission_level: Self::get_u8(&config_file, "function-permission-level", 2),
            generate_structures: Self::get_bool(&config_file, "generate-strucutre", true),
            hardcore: Self::get_bool(&config_file, "hardcore", false),
            hide_online_players: Self::get_bool(&config_file, "hide-online-players", false),
            initial_disabled_packs: Self::get_optional_string(
                &config_file,
                "initial-disabled-packs",
            ),
            initial_enabled_packs: Self::get_optional_string(&config_file, "initial-enabled-packs")
                .unwrap_or_else(|| "vanilla".to_string()),
            level_name: Self::get_optional_string(&config_file, "level-name"),
            level_type: config_file
                .get_property("level-type")
                .unwrap_or("normal")
                .parse::<WorldPreset>()
                .unwrap_or(WorldPreset::Normal),
            log_ips: Self::get_bool(&config_file, "log-ips", true), // this one is obsured
            max_chained_neighbor_updates: Self::get_optional_i32(
                &config_file,
                "max-chained-neighbor-updates",
            ),
            max_tick_time: Self::get_i64(&config_file, "max-tick-time", 60000),
            max_world_size: Self::get_u32(&config_file, "max-world-size", 29999984),
            motd: Self::get_optional_string(&config_file, "motd"),
            network_compression_threshold: Self::get_i32(
                &config_file,
                "network-compression-threshold",
                256,
            ),
            online_mode: Self::get_bool(&config_file, "online-mode", true),
            op_permission_level: Self::get_u8(&config_file, "op-permission-level", 4),
            player_idle_timeout: Self::get_i32(&config_file, "player-idle-timeout", 0),
            prevent_proxy_connections: Self::get_bool(
                &config_file,
                "prevent-proxy-connections",
                false,
            ),
            pvp: Self::get_bool(&config_file, "pvp", true),
            query_port: Self::get_u16(&config_file, "query-port", 25565),
            rate_limit: Self::get_u32(&config_file, "rate-limit", 0),
            rcon_password: Self::get_optional_string(&config_file, "rcon.password"),
            require_resource_pack: Self::get_bool(&config_file, "require-ressource-pack", false),
            resource_pack: Self::get_optional_string(&config_file, "resource-pack"),
            resource_pack_prompt: Self::get_optional_string(&config_file, "resource-pack-prompt"),
            resource_pack_sha1: Self::get_optional_string(&config_file, "ressource-pack-sha1"),
            simulation_distance: Self::get_u8(&config_file, "simulation-distance", 10),
            spawn_animals: Self::get_bool(&config_file, "spawn-animals", true),
            spawn_monsters: Self::get_bool(&config_file, "spawn-monster", true),
            spawn_npcs: Self::get_bool(&config_file, "spawn-npcs", true),
            spawn_protection: Self::get_u16(&config_file, "spawn-protection", 16),
            sync_chunk_writes: Self::get_bool(&config_file, "sync-chunk-writes", false),
            use_native_transport: Self::get_bool(&config_file, "use-native-transport", true),
            view_distance: Self::get_u8(&config_file, "view-distance", 10),
            white_list: Self::get_bool(&config_file, "white-list", false),
        }
    }
    fn get_bool(config: &Properties, key: &str, default: bool) -> bool {
        config
            .get_property(key)
            .unwrap_or(default.to_string().as_str())
            .parse()
            .unwrap_or(default)
    }
    fn get_u16(config: &Properties, key: &str, default: u16) -> u16 {
        config
            .get_property(key)
            .unwrap_or(default.to_string().as_str())
            .parse()
            .unwrap_or(default)
    }
    fn get_u32(config: &Properties, key: &str, default: u32) -> u32 {
        config
            .get_property(key)
            .unwrap_or(default.to_string().as_str())
            .parse()
            .unwrap_or(default)
    }
    fn get_u8(config: &Properties, key: &str, default: u8) -> u8 {
        config
            .get_property(key)
            .unwrap_or(default.to_string().as_str())
            .parse()
            .unwrap_or(default)
    }
    fn get_i32(config: &Properties, key: &str, default: i32) -> i32 {
        config
            .get_property(key)
            .unwrap_or(default.to_string().as_str())
            .parse()
            .unwrap_or(default)
    }
    fn get_i64(config: &Properties, key: &str, default: i64) -> i64 {
        config
            .get_property(key)
            .unwrap_or(default.to_string().as_str())
            .parse()
            .unwrap_or(default)
    }
    fn get_enum<T: std::str::FromStr>(config: &Properties, key: &str, default: T) -> T {
        config
            .get_property(key)
            .unwrap_or("")
            .parse()
            .unwrap_or(default)
    }
    fn get_optional_string(config: &Properties, key: &str) -> Option<String> {
        config
            .get_property(key)
            .ok()
            .filter(|s| !s.is_empty())
            .map(String::from)
    }

    fn get_optional_i64(config: &Properties, key: &str) -> Option<i64> {
        config.get_property(key).ok().and_then(|s| s.parse().ok())
    }
    fn get_optional_i32(config: &Properties, key: &str) -> Option<i32> {
        config.get_property(key).ok().and_then(|s| s.parse().ok())
    }

    fn get_optional_ip(config: &Properties, key: &str) -> Option<Ipv4Addr> {
        config.get_property(key).ok().and_then(|s| s.parse().ok())
    }
}
