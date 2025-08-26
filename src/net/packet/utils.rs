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
pub fn name_of<T: ?Sized>(value: Option<&T>) -> &'static str {
    match value {
        None => type_name::<T>().rsplit("::").next().unwrap(),
        Some(v) => type_name_of_val(v).rsplit("::").next().unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Unit tests for get_hex_repr, get_bin_repr, get_dec_repr, and name_of

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_get_hex_repr_empty() {
        assert_eq!(get_hex_repr(&[]), "");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_get_hex_repr_single_zero() {
        assert_eq!(get_hex_repr(&[0x00]), "00");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_get_hex_repr_single_ff() {
        assert_eq!(get_hex_repr(&[0xFF]), "FF");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_get_hex_repr_mixed_sequence_uppercase_and_spacing() {
        let data = [0x0A, 0x1b, 0xC3, 0x00, 0xFF];
        // Expect uppercase hex, two digits each, single spaces between, no trailing space
        assert_eq!(get_hex_repr(&data), "0A 1B C3 00 FF");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_get_bin_repr_empty() {
        assert_eq!(get_bin_repr(&[]), "");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_get_bin_repr_leading_zeros_preserved() {
        assert_eq!(get_bin_repr(&[0x01]), "00000001");
        assert_eq!(get_bin_repr(&[0x00]), "00000000");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_get_bin_repr_mixed_sequence_spacing() {
        let data = [0xFF, 0xC3, 0x00];
        assert_eq!(get_bin_repr(&data), "11111111 11000011 00000000");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_get_dec_repr_empty() {
        assert_eq!(get_dec_repr(&[]), "");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_get_dec_repr_mixed_sequence_spacing() {
        let data = [0u8, 7u8, 127u8, 200u8, 255u8];
        assert_eq!(get_dec_repr(&data), "0 7 127 200 255");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_name_of_none_with_primitives() {
        assert_eq!(name_of::<u8>(None), "u8");
        assert_eq!(name_of::<u16>(None), "u16");
        assert_eq!(name_of::<[u8; 3]>(None), "[u8; 3]");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_name_of_some_with_primitives_and_alloc_types() {
        let a = 10u32;
        let s = String::from("hi");
        assert_eq!(name_of(Some(&a)), "u32");
        // Path prefixes like alloc:: or std:: are stripped by the function; expect the last segment only
        assert_eq!(name_of(Some(&s)), "String");
    }

    // Test authored by AI (GPT-5 Thinking)
    #[test]
    fn test_name_of_with_local_custom_type() {
        struct MyLocalType {
            _x: u8,
        }
        let v = MyLocalType { _x: 1 };
        assert_eq!(name_of::<MyLocalType>(None), "MyLocalType");
        assert_eq!(name_of(Some(&v)), "MyLocalType");
    }

    // // Test authored by AI (GPT-5 Thinking)
    // #[test]
    // fn test_name_of_trait_object_reports_dynamic_type() {
    //     use core::fmt::Display;
    //     let value: i32 = 7;
    //     let dyn_ref: &dyn Display = &value;
    //     // T is dyn Display here, but the function uses type_name_of_val so it should return the dynamic type "i32"
    //     assert_eq!(name_of(Some(dyn_ref)), "i32");
    // }
}
