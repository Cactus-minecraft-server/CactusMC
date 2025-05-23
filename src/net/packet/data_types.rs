use thiserror::Error;

/// Implementation of the LEB128 variable-length code compression algorithm.
/// Pseudo-code of this algorithm taken from https://wiki.vg/Protocol#VarInt_and_VarLong
/// A VarInt may not be longer than 5 bytes.
pub mod varint {
    use super::CodecError;

    const SEGMENT_BITS: i32 = 0x7F; // 0111 1111
    const CONTINUE_BIT: i32 = 0x80; // 1000 0000

    /// Tries to read a VarInt **beginning from the first byte of the data**, until either the
    /// VarInt is read or it exceeds 5 bytes and the function returns Err.
    pub fn read(data: &[u8]) -> Result<(i32, usize), CodecError> {
        let mut value: i32 = 0;
        let mut position: usize = 0;
        let mut length: usize = 0;

        // Iterate over each byte of `data` and cast as i32.
        for byte in data.iter().map(|&b| b as i32) {
            value |= (byte & SEGMENT_BITS) << position;
            length += 1;

            if (byte & CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            // Even though 5 * 7 = 35 bits would be correct,
            // we can't go past the input type (i32).
            if position >= 32 {
                return Err(CodecError::DecodeVarIntTooLong);
            }
        }

        if length == 0 {
            Err(CodecError::DecodeVarIntEmpty)
        } else {
            Ok((value, length))
        }
    }

    /// This function encodes a i32 to a Vec<u8>.
    /// The returned Vec<u8> may not be longer than 5 elements.
    pub fn write(mut value: i32) -> Vec<u8> {
        let mut result = Vec::<u8>::with_capacity(5);

        loop {
            let byte = (value & SEGMENT_BITS) as u8;

            // Moves the sign bit too by doing bitwise operation on the u32.
            value = ((value as u32) >> 7) as i32;

            // Value == 0 means that it's a positive value and it's been shifted enough.
            // Value == -1 means that it's a negative number.
            //
            // If value == 0, we've encoded all significant bits of a positive number
            // If value == -1, we've encoded all significant bits of a negative number
            if value == 0 || value == -1 {
                result.push(byte);
                break;
            } else {
                result.push(byte | CONTINUE_BIT as u8);
            }
        }

        result
    }
}

/// Implementation of the LEB128 variable-length compression algorithm.
/// Pseudo-code of this algorithm from https://wiki.vg/Protocol#VarInt_and_VarLong.
/// A VarLong may not be longer than 10 bytes.
pub mod varlong {
    use super::CodecError;

    const SEGMENT_BITS: i64 = 0x7F; // 0111 1111
    const CONTINUE_BIT: i64 = 0x80; // 1000 0000

    /// Tries to read a VarLong **beginning from the first byte of the data**, until either the
    /// VarLong is read or it exceeds 10 bytes and the function returns Err.
    pub fn read(data: &[u8]) -> Result<(i64, usize), CodecError> {
        let mut value: i64 = 0;
        let mut position: usize = 0;
        let mut length: usize = 0;

        // Iterate over each byte of `data` and cast as i64.
        for byte in data.iter().map(|&b| b as i64) {
            value |= (byte & SEGMENT_BITS) << position;
            length += 1;

            if (byte & CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            // Even though it might be 10 * 7 = 70 instead of 64.
            // The wiki says 64 :shrug:
            if position >= 64 {
                return Err(CodecError::DecodeVarLongTooLong);
            }
        }

        if length == 0 {
            Err(CodecError::DecodeVarLongEmpty)
        } else {
            Ok((value, length))
        }
    }

    /// This function encodes a i64 to a Vec<u8>.
    /// The returned Vec<u8> may not be longer than 10 elements.
    pub fn write(mut value: i64) -> Vec<u8> {
        let mut result = Vec::<u8>::with_capacity(10);

        loop {
            let byte = (value & SEGMENT_BITS) as u8;

            // Moves the sign bit too by doing bitwise operation on the u32.
            value = ((value as u64) >> 7) as i64;

            // Value == 0 means that it's a positive value and it's been shifted enough.
            // Value == -1 means that it's a negative number.
            //
            // If value == 0, we've encoded all significant bits of a positive number
            // If value == -1, we've encoded all significant bits of a negative number
            if value == 0 || value == -1 {
                result.push(byte);
                break;
            } else {
                result.push(byte | CONTINUE_BIT as u8);
            }
        }

        result
    }
}

// TODO: Maybe find a better way to do errors than having one error type per data type. This is
// smelly.

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    #[error("VarInt decoding error: value too long (max 5 bytes)")]
    DecodeVarIntTooLong,
    #[error("VarLong decoding error: value too long (max 10 bytes)")]
    DecodeVarLongTooLong,
    #[error("VarInt decoding error: no bytes read (min 1 byte)")]
    DecodeVarIntEmpty,
    #[error("VarLong decoding error: no bytes read (min 1 byte)")]
    DecodeVarLongEmpty,

    #[error("String decoding error: invalid VarInt")]
    DecodeString,
    #[error("String decoding error: invalid string length or invalid VarInt string byte size")]
    InvalidStringLength,
    #[error("String length error: string cannot be blank")]
    BlankString,
    #[error("String length error: string is too long")]
    InvalidEncoding,
}

/// Implementation of the String(https://wiki.vg/Protocol#Type:String).
/// It is a UTF-8 string prefixed with its size in bytes as a VarInt.
///
/// For instance, with &[6, 72, 69, 76, 76, 79, 33, 0xFF, 0xFF, 0xFF] the function
/// will return "HELLO!" and 0xFF are just garbage data, since the string is 6 bytes long,
/// the 0xFF are ignored.
pub mod string {
    use core::str;

    use log::info;

    use super::{varint, CodecError};

    // The maximum number of bytes the whole String (including the VarInt) can be.
    // 32767 is the max number of UTF-16 code units allowed. Multiplying by 3 accounts for
    // the maximum bytes a single UTF-8 code unit could occupy in UTF-8 encoding.
    // The +3 accounts for the maximum potential size of the VarInt that prefixes the string length.
    const MAX_UTF_16_UNITS: usize = 32767;
    const MAX_DATA_SIZE: usize = MAX_UTF_16_UNITS * 3 + 3;

    /// Tries to read a String **beginning from the first byte of the data**, until either the
    /// end of the String or error.
    ///
    /// If I understood, the VarInt at the beginning of the String is specifying the number of
    /// bytes the actual UTF-8 string takes in the packet. Then, we have to convert the bytes into
    /// an UTF-8 string, then convert it to UTF-16 to count the number of code points (also, code
    /// points above U+FFFF count as 2) to check if the String is following or not the rules.
    pub fn read(data: &[u8]) -> Result<(String, usize), CodecError> {
        match varint::read(data) {
            Ok(read_varint) => {
                let string_bytes_length: usize = read_varint.0 as usize;
                let read_bytes: usize = read_varint.1;

                // The position where the last string byte is.
                // string bytes size + string bytes
                let last_string_byte: usize = read_bytes + string_bytes_length;

                info!("Data: {data:?}");
                info!("Number of bytes of the length: {read_bytes}");
                info!("Number of bytes of the string: {string_bytes_length}");

                // If there are more bytes of string than the length of the data.
                if last_string_byte > data.len() {
                    return Err(CodecError::InvalidStringLength);
                }

                // If VarInt + String is greater than max allowed.
                if last_string_byte > MAX_DATA_SIZE {
                    return Err(CodecError::InvalidStringLength);
                }

                // We omit the first VarInt bytes and stop at the end of the string.
                let string_data = &data[read_bytes..last_string_byte];

                // Decode UTF-8 to a string
                let utf8_str: &str =
                    str::from_utf8(string_data).map_err(|_| CodecError::InvalidEncoding)?;

                // Convert the string to potentially UTF-16 units and count them
                let utf16_units = utf8_str.encode_utf16().count();

                // Check if the utf16_units exceed the allowed maximum
                if utf16_units > MAX_UTF_16_UNITS {
                    return Err(CodecError::InvalidStringLength);
                }

                Ok((utf8_str.to_string(), last_string_byte))
            }

            Err(_) => Err(CodecError::DecodeString),
        }

        //UTF-8 string prefixed with its size in bytes as a VarInt. Maximum length of n characters, which varies by context. The encoding used on the wire is regular UTF-8, not Java's "slight modification". However, the length of the string for purposes of the length limit is its number of UTF-16 code units, that is, scalar values > U+FFFF are counted as two. Up to n × 3 bytes can be used to encode a UTF-8 string comprising n code units when converted to UTF-16, and both of those limits are checked. Maximum n value is 32767. The + 3 is due to the max size of a valid length VarInt.
    }

    /// Writes a Protocol String from a &str.
    pub fn write(data: &str) -> Result<Vec<u8>, CodecError> {
        // Convert the string to potentially UTF-16 units and count them
        let utf16_units = data.encode_utf16().count();

        // Check if the utf16_units exceed the allowed maximum
        if utf16_units > MAX_UTF_16_UNITS {
            return Err(CodecError::InvalidStringLength);
        }

        // This is the lengh of the input &[str] which is a UTF-8 string.
        let mut string_length_varint: Vec<u8> = varint::write(data.len() as i32);

        // Pre-allocate exactly the number of bytes to have the VarInt and the String data.
        let mut result: Vec<u8> = Vec::with_capacity(data.len() + string_length_varint.len());

        // Add VarInt string length.
        result.append(&mut string_length_varint);
        // Add UTF-8 string bytes.
        result.extend_from_slice(data.as_bytes());

        if result.len() > MAX_DATA_SIZE {
            return Err(CodecError::InvalidStringLength);
        }

        Ok(result)
    }
}

/// Tests mostly written by AI, and not human-checked.
#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;
    use rand::Rng;
    use std::collections::HashMap;

    #[test]
    fn test_varint_read() {
        let values: HashMap<i32, Vec<u8>> = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (127, vec![0x7F]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xFF, 0x01]),
            (25565, vec![0xDD, 0xC7, 0x01]),
            (2097151, vec![0xFF, 0xFF, 0x7F]),
            (i32::MAX, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
            (i32::MIN, vec![0x80, 0x80, 0x80, 0x80, 0x08]),
        ]
        .iter()
        .cloned()
        .collect();

        for (expected_value, encoded) in values.iter() {
            let (decoded_value, decoded_length) = varint::read(encoded).unwrap();
            assert_eq!(decoded_value, *expected_value);
            assert_eq!(decoded_length, encoded.len());
        }
    }

    #[test]
    fn test_varint_write() {
        let values: HashMap<i32, Vec<u8>> = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (127, vec![0x7F]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xFF, 0x01]),
            (25565, vec![0xDD, 0xC7, 0x01]),
            (2097151, vec![0xFF, 0xFF, 0x7F]),
            (i32::MAX, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
            (i32::MIN, vec![0x80, 0x80, 0x80, 0x80, 0x08]),
        ]
        .iter()
        .cloned()
        .collect();

        for (value, expected_encoded) in values.iter() {
            let encoded = varint::write(*value);
            assert_eq!(encoded, *expected_encoded);
        }
    }

    #[test]
    fn test_varint_roundtrip() {
        // Test a range of values including edge cases
        let test_values = [
            i32::MIN,
            i32::MIN + 1,
            -1000000,
            -1,
            0,
            1,
            1000000,
            i32::MAX - 1,
            i32::MAX,
        ];
        for &value in &test_values {
            let encoded = varint::write(value);
            let (decoded, _) = varint::read(&encoded).unwrap();
            assert_eq!(value, decoded, "Roundtrip failed for value: {}", value);
        }

        // Test a range of random values
        let mut rng = rand::thread_rng();
        for _ in 0..10_000 {
            let value = rng.gen::<i32>();
            let encoded = varint::write(value);
            let (decoded, _) = varint::read(&encoded).unwrap();
            assert_eq!(
                value, decoded,
                "Roundtrip failed for random value: {}",
                value
            );
        }
    }

    #[test]
    fn test_varint_invalid_input() {
        // Test for a VarInt that's too long
        let too_long = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        assert!(matches!(
            varint::read(&too_long),
            Err(CodecError::DecodeVarIntTooLong)
        ));
    }

    #[test]
    fn test_varlong_read() {
        let values: HashMap<i64, Vec<u8>> = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (127, vec![0x7F]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xFF, 0x01]),
            (25565, vec![0xDD, 0xC7, 0x01]),
            (2097151, vec![0xFF, 0xFF, 0x7F]),
            (
                i64::MAX,
                vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
            ),
            (
                -1,
                vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
            ),
            (
                i64::MIN,
                vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
            ),
        ]
        .iter()
        .cloned()
        .collect();

        for (expected_value, encoded) in values.iter() {
            let (decoded_value, decoded_length) = varlong::read(encoded).unwrap();
            assert_eq!(decoded_value, *expected_value);
            assert_eq!(decoded_length, encoded.len());
        }
    }

    #[test]
    fn test_varlong_write() {
        let values: HashMap<i64, Vec<u8>> = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (127, vec![0x7F]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xFF, 0x01]),
            (25565, vec![0xDD, 0xC7, 0x01]),
            (2097151, vec![0xFF, 0xFF, 0x7F]),
            (
                i64::MAX,
                vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
            ),
            (
                -1,
                vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
            ),
            (
                i64::MIN,
                vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
            ),
        ]
        .iter()
        .cloned()
        .collect();

        for (value, expected_encoded) in values.iter() {
            let encoded = varlong::write(*value);
            assert_eq!(encoded, *expected_encoded);
        }
    }

    #[test]
    fn test_varlong_roundtrip() {
        // Test a range of values including edge cases
        let test_values = [
            i64::MIN,
            i64::MIN + 1,
            -1000000000000,
            -1,
            0,
            1,
            1000000000000,
            i64::MAX - 1,
            i64::MAX,
        ];

        for &value in &test_values {
            let encoded = varlong::write(value);
            let (decoded, _) = varlong::read(&encoded).unwrap();
            assert_eq!(value, decoded, "Roundtrip failed for value: {}", value);
        }

        // Test a range of random values
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..10_000 {
            let value = rng.gen::<i64>();
            let encoded = varlong::write(value);
            let (decoded, _) = varlong::read(&encoded).unwrap();
            assert_eq!(
                value, decoded,
                "Roundtrip failed for random value: {}",
                value
            );
        }
    }

    #[test]
    fn test_varlong_invalid_input() {
        // Test for a VarLong that's too long
        let too_long = vec![
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01,
        ];
        assert!(matches!(
            varlong::read(&too_long),
            Err(CodecError::DecodeVarLongTooLong)
        ));
    }

    #[test]
    fn test_string_read_valid_ascii() {
        let s = "HELLO";
        let string_bytes = s.as_bytes();
        let length = string_bytes.len();

        // Encode length as VarInt
        let mut data = varint::write(length as i32);
        data.extend_from_slice(string_bytes);

        // Call string::read
        match string::read(&data) {
            Ok(result) => assert_eq!(result.0, s),
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_string_read_valid_utf8() {
        let s = "こんにちは"; // Japanese for "Hello"
        let string_bytes = s.as_bytes();
        let length = string_bytes.len();

        // Encode length as VarInt
        let mut data = varint::write(length as i32);
        data.extend_from_slice(string_bytes);

        // Call string::read
        match string::read(&data) {
            Ok(result) => assert_eq!(result.0, s),
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_string_read_blank_string() {
        let s = "";
        let string_bytes = s.as_bytes();
        let length = string_bytes.len(); // Should be 0

        // Encode length as VarInt
        let mut data = varint::write(length as i32);
        data.extend_from_slice(string_bytes);

        // Call string::read
        match string::read(&data) {
            Ok(val) => assert!(val.0.is_empty(), "Expected empty string, got {}", val.0),
            Err(e) => panic!("Expected Ok(), got {e}"),
        }
    }

    #[test]
    fn test_string_read_too_long_string() {
        // Assuming the maximum allowed length is 32767 bytes
        let max_allowed_length = 32767;

        // Create a string longer than the maximum allowed length
        let s = "A".repeat(max_allowed_length + 1);
        let string_bytes = s.as_bytes();
        let length = string_bytes.len();

        // Encode length as VarInt
        let mut data = varint::write(length as i32);
        println!("{data:?}");
        data.extend_from_slice(string_bytes);

        // Call string::read
        match string::read(&data) {
            Ok(_) => panic!("Expected StringTooLong error, but got Ok"),
            Err(e) => assert_eq!(e, CodecError::InvalidStringLength),
        }
    }

    #[test]
    fn test_string_read_invalid_varint() {
        // Create an invalid VarInt (6 bytes long, exceeding the 5-byte limit)
        let invalid_varint = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        let string_bytes = b"HELLO";

        let mut data = invalid_varint;
        data.extend_from_slice(string_bytes);

        // Call string::read
        match string::read(&data) {
            Ok(_) => panic!("Expected DecodeString error, but got Ok"),
            Err(e) => assert_eq!(e, CodecError::DecodeString),
        }
    }

    #[test]
    fn test_string_read_invalid_utf8() {
        // Valid VarInt for length 3
        let length = 3;
        let mut data = varint::write(length as i32);

        // Invalid UTF-8 bytes
        let invalid_utf8 = vec![0xFF, 0xFF, 0xFF];
        data.extend_from_slice(&invalid_utf8);

        // Call string::read
        match string::read(&data) {
            Ok(_) => panic!("Expected InvalidEncoding error, but got Ok"),
            Err(e) => assert_eq!(e, CodecError::InvalidEncoding),
        }
    }

    #[test]
    fn test_string_read_incomplete_data() {
        // Valid VarInt for length 10
        let length = 10;
        let mut data = varint::write(length as i32);

        // String data shorter than declared length
        let string_bytes = b"HELLO"; // Only 5 bytes
        data.extend_from_slice(string_bytes);

        // Call string::read
        match string::read(&data) {
            Ok(_) => panic!("Expected InvalidStringLength error, but got Ok"),
            Err(e) => assert_eq!(e, CodecError::InvalidStringLength),
        }
    }

    #[test]
    fn test_string_read_no_data() {
        // Valid VarInt for length 5
        let length = 5;
        let data = varint::write(length as i32); // No string data appended

        // Call string::read
        match string::read(&data) {
            Ok(_) => panic!("Expected InvalidStringLength error, but got Ok"),
            Err(e) => assert_eq!(e, CodecError::InvalidStringLength),
        }
    }

    #[test]
    fn test_string_read_empty_data() {
        let data: Vec<u8> = Vec::new();

        // Call string::read
        match string::read(&data) {
            Ok(_) => panic!("Expected DecodeString error, but got Ok"),
            Err(e) => assert_eq!(e, CodecError::DecodeString),
        }
    }

    #[test]
    fn test_string_read_random_strings() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            // Generate a random length between 1 and 100
            let length = rng.gen_range(1..=100);

            // Generate a random string of that length
            let s: String = (0..length)
                .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
                .collect();
            let string_bytes = s.as_bytes();

            // Encode length as VarInt
            let mut data = varint::write(string_bytes.len() as i32);
            data.extend_from_slice(string_bytes);

            // Call string::read
            match string::read(&data) {
                Ok(result) => assert_eq!(result.0, s),
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
    }

    #[test]
    fn test_write_valid_string() {
        let input = "Hello, World!";
        let expected_varint = varint::write(input.len() as i32);
        let expected_bytes = [expected_varint.as_slice(), input.as_bytes()].concat();

        let result = string::write(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_bytes);
    }

    #[test]
    fn test_write_empty_string() {
        let input = "";
        let expected_varint = varint::write(input.len() as i32);
        let expected_bytes = expected_varint;

        let result = string::write(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_bytes);
    }

    #[test]
    fn test_write_string_exceeding_max_utf16_units() {
        // Generate a string with a length greater than the maximum UTF-16 units allowed.
        let input: String = std::iter::repeat('𠀋').take(32768).collect(); // This character (U+0800B) uses 2 UTF-16 units.
        let result = string::write(&input);
        assert!(matches!(result, Err(CodecError::InvalidStringLength)));
    }

    #[test]
    fn test_write_string_exceeding_max_data_size() {
        // Generate a string that, when encoded with VarInt, would exceed the max data size.
        let long_string = "a".repeat(32767 * 3 + 4); // Over the size limit after accounting for VarInt size.
        let result = string::write(&long_string);
        assert!(matches!(result, Err(CodecError::InvalidStringLength)));
    }

    #[test]
    fn test_write_string_with_special_characters() {
        let input = "こんにちは、世界! 🌍"; // Includes Unicode characters and an emoji.
        let expected_varint = varint::write(input.len() as i32);
        let expected_bytes = [expected_varint.as_slice(), input.as_bytes()].concat();

        let result = string::write(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_bytes);
    }

    #[test]
    fn test_write_string_near_max_length() {
        let max_length = 32767; // Maximum number of UTF-16 code units.
        let input: String = std::iter::repeat('a').take(max_length).collect(); // Each 'a' is one UTF-16 unit.
        let expected_varint = varint::write(input.len() as i32);
        let expected_bytes = [expected_varint.as_slice(), input.as_bytes()].concat();

        let result = string::write(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_bytes);
    }

    #[test]
    fn test_write_to_read_loop() {
        let input = "こんにちは、世界! 🌍"; // Includes Unicode characters and an emoji.

        let converted = string::write(input);
        assert!(converted.is_ok());

        match converted {
            Ok(bytes) => match string::read(&bytes) {
                Ok(string) => {
                    assert!(
                        input == string.0,
                        "input != string: input='{input}' and string='{}'",
                        string.0
                    );
                }
                Err(e) => {
                    panic!("Error converting string: {e}");
                }
            },
            Err(e) => {
                panic!("Error converting string: {e}");
            }
        }
    }
}
