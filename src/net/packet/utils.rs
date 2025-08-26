//! This module exposes some utility functions, mainly for creating visual things.

use std::any::{type_name, type_name_of_val};

/// Returns the hexadecimal representation of bytes.
/// E.g., 65535 -> 0xFF 0xFF
pub fn get_hex_repr(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Returns the binary representation of bytes.
/// E.g., 65535 -> 11111111 11111111
pub fn get_bin_repr(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:08b}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Returns the decimal representation of bytes.
pub fn get_dec_repr(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Returns the name of a type as a string slice,
/// or the name of the type of value.
pub fn name_of<T>(value: Option<&T>) -> &'static str {
    match value {
        None => type_name::<T>().rsplit("::").next().unwrap(),
        Some(v) => type_name_of_val(v).rsplit("::").next().unwrap(),
    }
}
