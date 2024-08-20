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

// TODO: Finish writing unit-tests

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_encode_varint() {
        let mut values: HashMap<i32, Vec<u8>> = HashMap::new();

        values.insert(0, vec![0x00]);
        values.insert(1, vec![0x01]);
        values.insert(127, vec![0x7F]);
        values.insert(128, vec![0x80, 0x01]);
        values.insert(255, vec![0xFF, 0x01]);
        values.insert(25565, vec![0xDD, 0xC7, 0x01]);
        values.insert(2097151, vec![0xFF, 0xFF, 0x7F]);
        values.insert(2147483647, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07]);
        values.insert(-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        values.insert(-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08]);

        for (value, bytes) in values {
            let encoded_varint: Vec<u8> = encode::varint(value).expect("Failed to encode varint");
            assert_eq!(encoded_varint, bytes);
        }
    }
}
