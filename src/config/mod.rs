//! This module is the interface between the server.properties file. Querying for server settings.
// !TODO generator_settings
// !Todo text-filtering-config
use dot_properties::{read_properties, Properties};
use std::fs::File;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;

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
enum Difficulty {
    EASY,
    NORMAL,
    HARD,
}
enum Gamemode{
    ADVENTURE,
    SURVIVAL,
    CREATIVE,
    SPECTATOR,
}
enum WorlPreset{
    NORMAL,
    FLAT,
    LARGEBIOMES,
    AMPLIFIED,
    SINGLEBIOMESURFACE,
}

struct Settings{
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
    server_port: u16,
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

pub fn read(filepath: &Path) -> std::io::Result<Properties> { //do not forget to remove the pub when function new is finish!
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    read_properties(&mut reader).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}


impl Settings{
    pub fn new(filepath: &Path) -> Self{
         let config_file = read(Path::new(crate::consts::filepaths::PROPERTIES)).expect("Error reading server.properties file");

        Self{
            enable_jmx_monitoring: config_file.get_property("enable_jmx_monitoring").unwrap().parse::<bool>().unwrap(),
            rcon_port: config_file.get_property("rcon_port").unwrap().parse::<u16>().unwrap(),
            level_seed: match config_file.get_property("level_seed").unwrap() {
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
            enable_command_block: config_file.get_property("enable_command_block").unwrap().parse::<bool>().unwrap(),
            enable_query: config_file.get_property("enable_query").unwrap().parse::<bool>().unwrap(),
            enforce_secure_profile: config_file.get_property("enforce_secure_profile").unwrap().parse::<bool>().unwrap(),
            level_name: match config_file.get_property("level_name").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            motd: match config_file.get_property("motd").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            query_port: config_file.get_property("enable_command_block").unwrap().parse::<u16>().unwrap(),
            pvp: config_file.get_property("pvp").unwrap().parse::<bool>().unwrap(),
            generate_structures: config_file.get_property("generate_structures").unwrap().parse::<bool>().unwrap(),
            max_chained_neighbor_updates: match config_file.get_property("motd").unwrap() {
                "" => None,
                s => Some(s.parse::<i32>().unwrap()),
            },
            difficulty: match config_file.get_property("difficulty").unwrap() {
                "normal" => Difficulty::NORMAL,
                "easy" => Difficulty::EASY,
                "hard" => Difficulty::HARD,
                _ => Difficulty::EASY, // default value
            },
            network_compression_threshold: config_file.get_property("network_compression_threshold").unwrap().parse::<i32>().unwrap(),
            max_tick_time: config_file.get_property("max_tick_time").unwrap().parse::<i64>().unwrap(),
            require_resource_pack: config_file.get_property("require_resource_pack").unwrap().parse::<bool>().unwrap(),
            use_native_transport: config_file.get_property("use_native_transport").unwrap().parse::<bool>().unwrap(),
            max_players: config_file.get_property("use_native_transport").unwrap().parse::<u32>().unwrap(),
            online_mode: config_file.get_property("online_mode").unwrap().parse::<bool>().unwrap(),
            enable_status: config_file.get_property("enable_status").unwrap().parse::<bool>().unwrap(),
            allow_flight: config_file.get_property("allow_flight").unwrap().parse::<bool>().unwrap(),
            initial_disabled_packs: match config_file.get_property("initial_disabled_packs").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            broadcast_rcon_to_ops: config_file.get_property("broadcast_rcon_to_ops").unwrap().parse::<bool>().unwrap(),
            view_distance: config_file.get_property("view_distance").unwrap().parse::<u8>().unwrap(),
            server_ip: match config_file.get_property("server_ip").unwrap() {
                "" => None,
                s => Some(s.parse::<Ipv4Addr>().unwrap()),
            },
            resource_pack_prompt: match config_file.get_property("resource_pack_prompt").unwrap() {
                "" => None,
                s => Some(s.parse::<String>().unwrap()),
            },
            allow_nether: todo!(),
            server_port: todo!(),
            enable_rcon: todo!(),
            sync_chunk_writes: todo!(),
            op_permission_level: todo!(),
            prevent_proxy_connections: todo!(),
            hide_online_players: todo!(),
            resource_pack: todo!(),
            entity_broadcast_range_percentage: todo!(),
            simulation_distance: todo!(),
            rcon_password: todo!(),
            player_idle_timeout: todo!(),
            force_gamemode: todo!(),
            rate_limit: todo!(),
            hardcore: todo!(),
            white_list: todo!(),
            broadcast_console_to_ops: todo!(),
            spawn_npcs: todo!(),
            spawn_animals: todo!(),
            log_ips: todo!(),
            function_permission_level: todo!(),
            initial_enabled_packs: todo!(),
            level_type: todo!(),
            spawn_monsters: todo!(),
            enforce_whitelist: todo!(),
            spawn_protection: todo!(),
            resource_pack_sha1: todo!(),
            max_world_size: todo!(),
            //generator_settings: todo!(),
            //text_filtering_config: todo!(),


        }

    }
    //fn gamemode_to_enum(inp)
}
