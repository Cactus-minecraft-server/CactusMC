use crate::fs_manager;
use clap::Parser;
use log::error;

#[derive(Parser)]
#[command(name = "CopperMC")]
#[command(about = "This is the about, please change", long_about = None)]
struct Cli {
    /// Removes all files related to the server, excluding the server.
    #[arg(short, long)]
    remove_files: Option<bool>,
}

/// Retrieves args and initializes the argument parsing logic.
pub fn init() {
    let args = Cli::parse();

    parse_args(args);
}

/// Parses args and calls the appropriate functions.
fn parse_args(args: Cli) {
    if let Some(_) = args.remove_files {
        if let Err(e) = fs_manager::clean_files() {
            error!("Error(s) when cleaning files: {e}");
        }
    }
}
