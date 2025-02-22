use log::info;

use crate::consts::messages;

/// Prints the starting greetings
pub fn greet() {
    info!("{}", *messages::GREET);
}
