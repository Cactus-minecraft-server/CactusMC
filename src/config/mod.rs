//! This module is the interface between the server.properties file. Querying for server settings.
// !TODO generator_settings
// !Todo text-filtering-config
// use dot_properties::{read_properties, Properties};
use std::fs::File;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::net::Ipv4Addr;
use std::path::Path;

use read_properties::Properties;
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

impl Settings {
    pub fn new() -> Self {
        let config_file = read(Path::new(crate::consts::file_paths::PROPERTIES))
            .expect("Error reading {server.properties} file");

        Self {
            enable_jmx_monitoring: config_file
                .get_property("enable-jmx-monitoring")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            rcon_port: config_file
                .get_property("rcon.port")
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            level_seed: match config_file.get_property("level-seed").unwrap() {
                "" => None,
                s => Some(s.parse::<i64>().unwrap()),
            },
            gamemode: match config_file
                .get_property("gamemode")
                .unwrap()
                .to_lowercase()
                .as_str()
            {
                "creative" => Gamemode::Creative,
                "survival" => Gamemode::Survival,
                "spectator" => Gamemode::Spectator,
                "adventure" => Gamemode::Adventure,
                _ => Gamemode::Survival, // default value
            },
            enable_command_block: config_file
                .get_property("enable-command-block")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            enable_query: config_file
                .get_property("enable-query")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            enforce_secure_profile: config_file
                .get_property("enforce-secure-profile")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            level_name: match config_file.get_property("level-name").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            motd: match config_file.get_property("motd").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            query_port: config_file
                .get_property("query.port")
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            pvp: config_file
                .get_property("pvp")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            generate_structures: config_file
                .get_property("generate-structures")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            max_chained_neighbor_updates: match config_file
                .get_property("max-chained-neighbor-updates")
                .unwrap()
            {
                "" => None,
                s => Some(s.parse::<i32>().unwrap()),
            },
            difficulty: match config_file.get_property("difficulty").unwrap() {
                "normal" => Difficulty::Normal,
                "easy" => Difficulty::Easy,
                "hard" => Difficulty::Hard,
                _ => Difficulty::Easy, // default value
            },
            network_compression_threshold: config_file
                .get_property("network-compression-threshold")
                .unwrap()
                .parse::<i32>()
                .unwrap(),
            max_tick_time: config_file
                .get_property("max-tick-time")
                .unwrap()
                .parse::<i64>()
                .unwrap(),
            require_resource_pack: config_file
                .get_property("require-resource-pack")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            use_native_transport: config_file
                .get_property("use-native-transport")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            max_players: config_file
                .get_property("max-players")
                .unwrap()
                .parse::<u32>()
                .unwrap(),
            online_mode: config_file
                .get_property("online-mode")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            enable_status: config_file
                .get_property("enable-status")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            allow_flight: config_file
                .get_property("allow-flight")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            initial_disabled_packs: match config_file
                .get_property("initial-disabled-packs")
                .unwrap()
            {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            broadcast_rcon_to_ops: config_file
                .get_property("broadcast-rcon-to-ops")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            view_distance: config_file
                .get_property("view-distance")
                .unwrap()
                .parse::<u8>()
                .unwrap(),
            server_ip: match config_file.get_property("server-ip").unwrap() {
                "" => None,
                s => Some(s.parse::<Ipv4Addr>().unwrap()),
            },
            resource_pack_prompt: match config_file.get_property("resource-pack-prompt").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            allow_nether: config_file
                .get_property("allow-nether")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            server_port: config_file
                .get_property("server-port")
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            enable_rcon: config_file
                .get_property("enable-rcon")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            sync_chunk_writes: config_file
                .get_property("sync-chunk-writes")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            op_permission_level: config_file
                .get_property("op-permission-level")
                .unwrap()
                .parse::<u8>()
                .unwrap(),
            prevent_proxy_connections: config_file
                .get_property("prevent-proxy-connections")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            hide_online_players: config_file
                .get_property("hide-online-players")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            resource_pack: match config_file.get_property("resource-pack").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            entity_broadcast_range_percentage: config_file
                .get_property("entity-broadcast-range-percentage")
                .unwrap()
                .parse::<u8>()
                .unwrap(),
            simulation_distance: config_file
                .get_property("simulation-distance")
                .unwrap()
                .parse::<u8>()
                .unwrap(),
            rcon_password: match config_file.get_property("rcon.password").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            player_idle_timeout: config_file
                .get_property("simulation-distance")
                .unwrap()
                .parse::<i32>()
                .unwrap(),
            force_gamemode: config_file
                .get_property("force-gamemode")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            rate_limit: config_file
                .get_property("simulation-distance")
                .unwrap()
                .parse::<u32>()
                .unwrap(),
            hardcore: config_file
                .get_property("hardcore")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            white_list: config_file
                .get_property("white-list")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            broadcast_console_to_ops: config_file
                .get_property("broadcast-console-to-ops")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            spawn_npcs: config_file
                .get_property("spawn-npcs")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            spawn_animals: config_file
                .get_property("spawn-animals")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            log_ips: config_file
                .get_property("log-ips")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            function_permission_level: config_file
                .get_property("function-permission-level")
                .unwrap()
                .parse::<u8>()
                .unwrap(),
            initial_enabled_packs: config_file
                .get_property("initial-enabled-packs")
                .unwrap()
                .parse::<String>()
                .unwrap(),
            // level-type and also be "minecraft\:normal"
            level_type: match config_file.get_property("level-type").unwrap() {
                "normal" => WorldPreset::Normal,
                "flat" => WorldPreset::Flat,
                "large_biomes" => WorldPreset::LargeBiomes,
                "amplified" => WorldPreset::SingleBiomeSurface,
                "single_biome_surface" => WorldPreset::Amplified,
                _ => WorldPreset::Normal, // default value
            },
            spawn_monsters: config_file
                .get_property("spawn-monsters")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            enforce_whitelist: config_file
                .get_property("enforce-whitelist")
                .unwrap()
                .parse::<bool>()
                .unwrap(),
            spawn_protection: config_file
                .get_property("spawn-protection")
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            resource_pack_sha1: match config_file.get_property("resource-pack-sha1").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            max_world_size: config_file
                .get_property("max-world-size")
                .unwrap()
                .parse::<u32>()
                .unwrap(),
            //generator_settings: todo!(),
            //text_filtering_config: todo!(),
        }
    }
    //fn gamemode_to_enum(inp)
}

