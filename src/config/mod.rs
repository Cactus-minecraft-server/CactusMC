//! This module is the interface between the server.properties file. Querying for server settings.
// !TODO generator_settings
// !Todo text-filtering-config
use dot_properties::{read_properties, Properties};
use std::fs::File;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::net::Ipv4Addr;
use std::path::Path;
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
pub enum Difficulty {
    EASY,
    NORMAL,
    HARD,
}
pub enum Gamemode {
    ADVENTURE,
    SURVIVAL,
    CREATIVE,
    SPECTATOR,
}
pub enum WorlPreset {
    NORMAL,
    FLAT,
    LARGEBIOMES,
    AMPLIFIED,
    SINGLEBIOMESURFACE,
}

pub struct Settings {
    enable_jmx_monitoring: bool,
    rcon_port: u16,
    level_seed: Option<i64>,
    gamemode: Gamemode,
    enable_command_block: bool,
    enable_query: bool,
    enforce_secure_profile: bool,
    level_name: Option<String>,
    motd: Option<String>,
    query_port: u16,
    pvp: bool,
    generate_structures: bool,
    max_chained_neighbor_updates: Option<i32>,
    difficulty: Difficulty,
    network_compression_threshold: i32,
    max_tick_time: i64,
    require_resource_pack: bool,
    use_native_transport: bool,
    max_players: u32,
    online_mode: bool,
    enable_status: bool,
    allow_flight: bool,
    initial_disabled_packs: Option<String>,
    broadcast_rcon_to_ops: bool,
    view_distance: u8,
    server_ip: Option<Ipv4Addr>,
    resource_pack_prompt: Option<String>,
    allow_nether: bool,
    pub server_port: u16,
    enable_rcon: bool,
    sync_chunk_writes: bool,
    op_permission_level: u8,
    prevent_proxy_connections: bool,
    hide_online_players: bool,
    resource_pack: Option<String>,
    entity_broadcast_range_percentage: u8,
    simulation_distance: u8,
    rcon_password: Option<String>,
    player_idle_timeout: i32,
    force_gamemode: bool,
    rate_limit: u32,
    hardcore: bool,
    white_list: bool,
    broadcast_console_to_ops: bool,
    spawn_npcs: bool,
    spawn_animals: bool,
    log_ips: bool,
    function_permission_level: u8,
    initial_enabled_packs: String,
    level_type: WorlPreset,
    spawn_monsters: bool,
    enforce_whitelist: bool,
    spawn_protection: u16,
    resource_pack_sha1: Option<String>,
    max_world_size: u32,
    //generator_settings:todo!(),
    //text_filtering_config:todo!(),
}

pub fn read(filepath: &Path) -> std::io::Result<Properties> {
    //do not forget to remove the pub while "new" function  is finish!
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    read_properties(&mut reader).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}

impl Settings {
    pub fn new() -> Self {
        let config_file = read(Path::new(crate::consts::filepaths::PROPERTIES))
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
            gamemode: match config_file.get_property("gamemode").unwrap() {
                "creative" => Gamemode::CREATIVE,
                "survival" => Gamemode::SURVIVAL,
                "spectator" => Gamemode::SPECTATOR,
                "adventure" => Gamemode::ADVENTURE,
                _ => Gamemode::SURVIVAL, // default value
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
                "normal" => Difficulty::NORMAL,
                "easy" => Difficulty::EASY,
                "hard" => Difficulty::HARD,
                _ => Difficulty::EASY, // default value
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
                "normal" => WorlPreset::NORMAL,
                "flat" => WorlPreset::FLAT,
                "large_biomes" => WorlPreset::LARGEBIOMES,
                "amplified" => WorlPreset::SINGLEBIOMESURFACE,
                "single_biome_surface" => WorlPreset::AMPLIFIED,
                _ => WorlPreset::NORMAL, // default value
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
