//! This module implements some utility functions.

/// Prints the hexadecimal representation of a slice of u8 to stdout.
fn print_hex_repr(data: &[u8]) {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ");
}

fn print_bool_repr(data: &[u8]) {
    data.iter().map(|b| format!("{:0}"))
}
