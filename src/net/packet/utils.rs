//! This module implements some utility functions.

/// Takes in a slice of u8 and returns its String hexadecimal representation.
pub fn get_hex_repr(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Takes in a slice of u8 and returns its String binary representation.
pub fn get_bin_repr(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:08b}", b))
        .collect::<Vec<String>>()
        .join(" ")
}


pub fn get_dec_repr(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{}", b))
        .collect::<Vec<String>>()
        .join(" ")
}
