/// This sub-module contains various encoding algorithms.
pub mod encode {
    use integer_encoding::VarInt;

    /// Encodes a i32 to a Vec<u8> using the VarInt algorithm.
    pub fn varint(value: i32) -> Option<Vec<u8>> {
        let result = value.encode_var_vec();
        if result.len() > 5 {
            return None;
        }
        Some(result)
    }

    /// Encodes a i64 to a Vec<u8> using the VarInt algorithm.
    pub fn varlong(value: i64) -> Option<Vec<u8>> {
        let result = value.encode_var_vec();
        if result.len() > 10 {
            return None;
        }
        Some(result)
    }
}

/// This sub-module contains various decoding algorithms.
pub mod decode {
    use integer_encoding::VarInt;

    /// Decodes the first VarInt in a slice of bytes
    pub fn varint(data: &[u8]) -> Option<(i32, usize)> {
        i32::decode_var(data)
    }

    /// Decodes the first VarLong in a slice of bytes
    pub fn varlong(data: &[u8]) -> Option<(i64, usize)> {
        i64::decode_var(data)
    }
}

// TODO: Write unit-tests for this file, it shouldn't be very difficult.
