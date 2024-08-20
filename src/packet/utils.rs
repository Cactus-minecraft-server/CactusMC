//! This module implements some utility functions.

/// Prints the hexadecimal representation of a slice of u8 to stdout.
pub fn get_hex_repr(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn get_bin_repr(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:08b}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

pub mod print {
    use colored::Colorize;

    pub fn red(msg: &str) {
        println!("{}", msg.red());
    }

    pub fn green(msg: &str) {
        println!("{}", msg.bright_green());
    }

    pub fn blue(msg: &str) {
        println!("{}", msg.bright_blue());
    }
}
