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

        Ok((value, length))
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

        Ok((value, length))
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

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("VarInt decoding error: value too long (max 5 bytes)")]
    DecodeVarIntTooLong,
    #[error("VarLong decoding error: value too long (max 10 bytes)")]
    DecodeVarLongTooLong,
}

/// Tests mostly written by AI, and not human-checked.
#[cfg(test)]
mod tests {
    use super::*;
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
}
