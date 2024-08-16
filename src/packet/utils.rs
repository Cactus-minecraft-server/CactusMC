//! This module implements some utility functions.

/// Prints the hexadecimal representation of a slice of u8 to stdout.
pub fn print_hex_repr(data: &[u8]) -> String {
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
