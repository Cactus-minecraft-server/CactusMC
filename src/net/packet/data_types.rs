use core::str;

use log::debug;
use thiserror::Error;

pub trait Encodable: Sized {
    /// Creates an instance from the first data type from a byte slice.
    /// The input slice remains unmodified.
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError>;

    /// Creates an instance from the first data type from a byte slice.
    /// Read bytes are consumed. For instance, if you were to read an UnsignedByte on [1, 5, 4, 8],
    /// the buffer would then be [4, 8] after the function call.
    fn consume_from_bytes(bytes: &mut &[u8]) -> Result<Self, CodecError> {
        let instance = Self::from_bytes(&bytes)?;
        *bytes = &bytes[instance.len()..];
        Ok(instance)
    }

    type ValueInput;
    /// Creates an instance from a value.
    fn from_value(value: Self::ValueInput) -> Result<Self, CodecError>;

    /// Serializes the instance into bytes
    fn get_bytes(&self) -> &[u8];

    type ValueOutput;
    /// Returns the value represented by this instance
    fn get_value(&self) -> Self::ValueOutput;

    // Returns the length of the encoded data in bytes.
    fn len(&self) -> usize {
        self.get_bytes().len()
    }
}

/// Represents datatypes in errors
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum DataType {
    VarInt,
    VarLong,
    StringProtocol,
    UnsignedShort,
    Uuid,
    Array,
    Other(&'static str),
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::VarInt => write!(f, "VarInt"),
            DataType::VarLong => write!(f, "VarLong"),
            DataType::StringProtocol => write!(f, "String"),
            DataType::UnsignedShort => write!(f, "UnsignedShort"),
            DataType::Uuid => write!(f, "UUID"),
            DataType::Array => write!(f, "Array"),
            DataType::Other(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ErrorReason {
    ValueTooLarge,
    ValueTooSmall,
    ValueEmpty,
    InvalidFormat(String),
    /// Notably used for NextState decoding.
    UnknownValue(String),
}

impl std::fmt::Display for ErrorReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorReason::ValueTooLarge => write!(f, "Value too large"),
            ErrorReason::ValueTooSmall => write!(f, "Value too small"),
            ErrorReason::ValueEmpty => write!(f, "Value empty"),
            ErrorReason::InvalidFormat(reason) => write!(f, "Invalid format: {}", reason),
            ErrorReason::UnknownValue(info) => write!(f, "Unknown value: {}", info),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    #[error("Encoding error for {0}: {1}")]
    Encoding(DataType, ErrorReason),

    #[error("Decoding error for {0}: {1}")]
    Decoding(DataType, ErrorReason),
}

/// Implementation of the LEB128 variable-length code compression algorithm.
/// Pseudo-code of this algorithm taken from https://wiki.vg/Protocol#VarInt_and_VarLong
/// A VarInt may not be longer than 5 bytes.
#[derive(Debug, Default)]
pub struct VarInt {
    // We're storing both the value and bytes to avoid redundant conversions.
    value: i32,
    bytes: Vec<u8>,
}

impl VarInt {
    const SEGMENT_BITS: i32 = 0x7F; // 0111 1111
    const CONTINUE_BIT: i32 = 0x80; // 1000 0000

    /// Tries to read a VarInt **beginning from the first byte of the data**, until either the
    /// VarInt is read or it exceeds 5 bytes and the function returns Err.
    fn read<T: AsRef<[u8]>>(data: T) -> Result<(i32, usize), CodecError> {
        let mut value: i32 = 0;
        let mut position: usize = 0;
        let mut length: usize = 0;

        // Iterate over each byte of `data` and cast as i32.
        for byte in data.as_ref().iter().map(|&b| b as i32) {
            value |= (byte & Self::SEGMENT_BITS) << position;
            length += 1;

            if (byte & Self::CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            // Even though 5 * 7 = 35 bits would be correct,
            // we can't go past the input type (i32).
            if position >= 32 {
                return Err(CodecError::Decoding(
                    DataType::VarInt,
                    ErrorReason::ValueTooLarge,
                ));
            }
        }

        if length == 0 {
            Err(CodecError::Decoding(
                DataType::VarInt,
                ErrorReason::ValueEmpty,
            ))
        } else {
            Ok((value, length))
        }
    }

    /// This function encodes a i32 to a Vec<u8>.
    /// The returned Vec<u8> may not be longer than 5 elements.
    fn write(mut value: i32) -> Result<Vec<u8>, CodecError> {
        let mut result = Vec::<u8>::with_capacity(5);

        loop {
            let byte = (value & Self::SEGMENT_BITS) as u8;

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
                result.push(byte | Self::CONTINUE_BIT as u8);
            }
        }

        if result.len() > 5 {
            Err(CodecError::Encoding(
                DataType::VarInt,
                ErrorReason::ValueTooLarge,
            ))
        } else {
            Ok(result)
        }
    }
}

impl Encodable for VarInt {
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();
        let value: (i32, usize) = Self::read(data)?;
        Ok(Self {
            value: value.0,
            // Only the VarInt is kept. The rest of the buffer is not accounted for.
            bytes: data[..value.1].to_vec(),
        })
    }

    type ValueInput = i32;

    fn from_value(value: Self::ValueInput) -> Result<Self, CodecError> {
        Ok(Self {
            value,
            bytes: Self::write(value)?,
        })
    }

    fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    type ValueOutput = i32;

    /// Returns the value of the VarInt (i32)
    fn get_value(&self) -> Self::ValueOutput {
        self.value
    }

    fn len(&self) -> usize {
        self.get_bytes().len()
    }
}

/// Implementation of the LEB128 variable-length compression algorithm.
/// Pseudo-code of this algorithm from https://wiki.vg/Protocol#VarInt_and_VarLong.
/// A VarLong may not be longer than 10 bytes.
pub struct VarLong {
    // We're storing both the value and bytes to avoid redundant conversions.
    value: i64,
    bytes: Vec<u8>,
}

impl VarLong {
    const SEGMENT_BITS: i64 = 0x7F; // 0111 1111
    const CONTINUE_BIT: i64 = 0x80; // 1000 0000

    /// Tries to read a VarLong **beginning from the first byte of the data**, until either the
    /// VarLong is read or it exceeds 10 bytes and the function returns Err.
    fn read<T: AsRef<[u8]>>(data: T) -> Result<(i64, usize), CodecError> {
        let mut value: i64 = 0;
        let mut position: usize = 0;
        let mut length: usize = 0;

        // Iterate over each byte of `data` and cast as i64.
        for byte in data.as_ref().iter().map(|&b| b as i64) {
            value |= (byte & Self::SEGMENT_BITS) << position;
            length += 1;

            if (byte & Self::CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            // Even though it might be 10 * 7 = 70 instead of 64.
            // The wiki says 64 :shrug:
            if position >= 64 {
                return Err(CodecError::Decoding(
                    DataType::VarLong,
                    ErrorReason::ValueTooLarge,
                ));
            }
        }

        if length == 0 {
            Err(CodecError::Decoding(
                DataType::VarLong,
                ErrorReason::ValueEmpty,
            ))
        } else {
            Ok((value, length))
        }
    }

    /// This function encodes a i64 to a Vec<u8>.
    /// The returned Vec<u8> may not be longer than 10 elements.
    fn write(mut value: i64) -> Result<Vec<u8>, CodecError> {
        let mut result = Vec::<u8>::with_capacity(10);

        loop {
            let byte = (value & Self::SEGMENT_BITS) as u8;

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
                result.push(byte | Self::CONTINUE_BIT as u8);
            }
        }

        if result.len() > 10 {
            Err(CodecError::Encoding(
                DataType::VarLong,
                ErrorReason::ValueTooLarge,
            ))
        } else {
            Ok(result)
        }
    }
}

impl Encodable for VarLong {
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();
        let value: (i64, usize) = Self::read(data)?;
        Ok(Self {
            value: value.0,
            // Only the VarInt is kept. The rest of the buffer is not accounted for.
            bytes: data[..value.1].to_vec(),
        })
    }

    type ValueInput = i64;

    fn from_value(value: Self::ValueInput) -> Result<Self, CodecError> {
        Ok(Self {
            value,
            bytes: Self::write(value)?,
        })
    }

    fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    type ValueOutput = i64;

    fn get_value(&self) -> Self::ValueOutput {
        self.value
    }
}

/// Implementation of the String(https://wiki.vg/Protocol#Type:String).
/// It is a UTF-8 string prefixed with its size in bytes as a VarInt.
///
/// For instance, with &[6, 72, 69, 76, 76, 79, 33, 0xFF, 0xFF, 0xFF] the function
/// will return "HELLO!" and 0xFF are just garbage data, since the string is 6 bytes long,
/// the 0xFF are ignored.
#[derive(Debug)]
pub struct StringProtocol {
    string: String,
    bytes: Vec<u8>,
}

impl StringProtocol {
    // The maximum number of bytes the whole String (including the VarInt) can be.
    // 32767 is the max number of UTF-16 code units allowed. Multiplying by 3 accounts for
    // the maximum bytes a single UTF-8 code unit could occupy in UTF-8 encoding.
    // The +3 accounts for the maximum potential size of the VarInt that prefixes the string length.
    const MAX_UTF_16_UNITS: usize = 32767;
    const MAX_DATA_SIZE: usize = Self::MAX_UTF_16_UNITS * 3 + 3;

    /// Tries to read a String **beginning from the first byte of the data**, until either the
    /// end of the String or error.
    ///
    /// If I understood, the VarInt at the beginning of the String is specifying the number of
    /// bytes the actual UTF-8 string takes in the packet. Then, we have to convert the bytes into
    /// an UTF-8 string, then convert it to UTF-16 to count the number of code points (also, code
    /// points above U+FFFF count as 2) to check if the String is following or not the rules.
    fn read<T: AsRef<[u8]>>(data: T) -> Result<(String, usize), CodecError> {
        let varint = VarInt::from_bytes(&data)?;

        // The VarInt-decoded length in bytes of the String.
        let string_bytes_length: usize = varint.get_value() as usize;

        // The length in bytes of the Length VarInt.
        let varint_length: usize = varint.get_bytes().len();

        // The position where the last string byte is.
        // string bytes size + string bytes
        let last_string_byte: usize = varint_length + string_bytes_length;

        debug!("READING STRING BEGIN");
        debug!("Data: {:?}", &data.as_ref());
        debug!("Number of bytes of the length: {varint_length}");
        debug!("Number of bytes of the string: {string_bytes_length}");
        debug!("READING STRING END");

        // If there are more bytes of string than the length of the data.
        if last_string_byte > data.as_ref().len() {
            return Err(CodecError::Decoding(
                DataType::StringProtocol,
                ErrorReason::InvalidFormat(
                    "String length is greater than provided bytes".to_string(),
                ),
            ));
        }

        // If VarInt + String is greater than max allowed.
        if last_string_byte > Self::MAX_DATA_SIZE {
            return Err(CodecError::Decoding(
                DataType::StringProtocol,
                ErrorReason::ValueTooLarge,
            ));
        }

        // We omit the first VarInt bytes and stop at the end of the string.
        let string_data = &data.as_ref()[varint_length..last_string_byte];

        // Decode UTF-8 to a string
        let utf8_str: &str = str::from_utf8(string_data).map_err(|err| {
            CodecError::Decoding(
                DataType::StringProtocol,
                ErrorReason::InvalidFormat(format!("String UTF-8 decoding error: {err}")),
            )
        })?;

        // Convert the string to potentially UTF-16 units and count them
        let utf16_units = utf8_str.encode_utf16().count();

        // Check if the utf16_units exceed the allowed maximum
        if utf16_units > Self::MAX_UTF_16_UNITS {
            return Err(CodecError::Decoding(
                DataType::StringProtocol,
                ErrorReason::InvalidFormat("Too many UTF-16 code points".to_string()),
            ));
        }

        Ok((utf8_str.to_string(), last_string_byte))

        //UTF-8 string prefixed with its size in bytes as a VarInt. Maximum length of n characters, which varies by context. The encoding used on the wire is regular UTF-8, not Java's "slight modification". However, the length of the string for purposes of the length limit is its number of UTF-16 code units, that is, scalar values > U+FFFF are counted as two. Up to n × 3 bytes can be used to encode a UTF-8 string comprising n code units when converted to UTF-16, and both of those limits are checked. Maximum n value is 32767. The + 3 is due to the max size of a valid length VarInt.
    }

    /// Writes a Protocol String from a &str.
    fn write<T: AsRef<str>>(string: T) -> Result<Vec<u8>, CodecError> {
        // Convert the string to potentially UTF-16 units and count them
        let utf16_units = string.as_ref().encode_utf16().count();

        // Check if the utf16_units exceed the allowed maximum
        if utf16_units > Self::MAX_UTF_16_UNITS {
            return Err(CodecError::Encoding(
                DataType::StringProtocol,
                ErrorReason::InvalidFormat("Too many UTF-16 code points".to_string()),
            ));
        }

        // VarInt-encoded length of the input UTF-8 string.
        let mut string_length_varint: Vec<u8> = VarInt::from_value(string.as_ref().len() as i32)?
            .get_bytes()
            .to_vec();

        // Pre-allocate exactly the number of bytes to have the VarInt and the String data.
        let mut result: Vec<u8> =
            Vec::with_capacity(string.as_ref().len() + string_length_varint.len());

        // Add VarInt string length.
        result.append(&mut string_length_varint);
        // Add UTF-8 string bytes.
        result.extend_from_slice(string.as_ref().as_bytes());

        if result.len() > Self::MAX_DATA_SIZE {
            return Err(CodecError::Encoding(
                DataType::StringProtocol,
                ErrorReason::ValueTooLarge,
            ));
        }

        Ok(result)
    }
}

impl Encodable for StringProtocol {
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();
        let string: (String, usize) = Self::read(data)?;
        Ok(Self {
            string: string.0,
            // Only take take the string, no more.
            bytes: data[..string.1].to_vec(),
        })
    }

    type ValueInput = String;

    fn from_value(value: Self::ValueInput) -> Result<Self, CodecError> {
        Ok(Self {
            string: value.to_string(),
            bytes: Self::write(value)?,
        })
    }

    fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    type ValueOutput = String;

    fn get_value(&self) -> Self::ValueOutput {
        self.string.clone()
    }
}

/// Implementation of the Big Endian unsigned short as per the Protocol Wiki.
#[derive(Debug)]
pub struct UnsignedShort {
    value: u16,
    bytes: [u8; 2],
}

impl UnsignedShort {
    /// Reads the first two bytes of the provided data in Big Endian format.
    fn read<T: AsRef<[u8]>>(bytes: T) -> Result<u16, CodecError> {
        let data: &[u8] = bytes.as_ref();
        if data.len() < 2 {
            return Err(CodecError::Decoding(
                DataType::UnsignedShort,
                ErrorReason::ValueTooSmall,
            ));
        }

        Ok(u16::from_be_bytes([data[0], data[1]]))
    }

    /// Returns the Big Endian representation of an u16.
    fn write(value: u16) -> [u8; 2] {
        value.to_be_bytes()
    }
}

impl Encodable for UnsignedShort {
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();
        let value: u16 = Self::read(data)?;
        Ok(Self {
            value,
            bytes: value.to_be_bytes(),
        })
    }

    type ValueInput = u16;

    fn from_value(value: Self::ValueInput) -> Result<Self, CodecError> {
        Ok(Self {
            value,
            bytes: Self::write(value),
        })
    }

    fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    type ValueOutput = u16;

    fn get_value(&self) -> Self::ValueOutput {
        self.value
    }
}

/// Represents a UUID. Encoded as an unsigned 128-bit integer in the protocol:
/// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Type:UUID
#[derive(Debug)]
pub struct Uuid {
    value: u128,
    /// There are 16 bytes in a u128.
    bytes: [u8; 16],
}

impl Uuid {
    /// Reads the first 16 bytes of the provided data in Big Endian format.
    fn read<T: AsRef<[u8]>>(bytes: T) -> Result<u128, CodecError> {
        let data: &[u8] = bytes.as_ref();

        if data.len() < 16 {
            return Err(CodecError::Decoding(
                DataType::Uuid,
                ErrorReason::ValueTooSmall,
            ));
        }

        let uuid_bytes = data[0..16]
            .try_into()
            .map_err(|err: std::array::TryFromSliceError| {
                CodecError::Encoding(DataType::Uuid, ErrorReason::InvalidFormat(err.to_string()))
            })?;

        Ok(u128::from_be_bytes(uuid_bytes))
    }

    /// Returns the Big Endian representation of an u16.
    ///
    /// There are 16 bytes in a u128.
    fn write(value: u128) -> [u8; 16] {
        value.to_be_bytes()
    }
}

impl Encodable for Uuid {
    fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();

        let value: u128 = Self::read(data)?;
        let bytes_: [u8; 16] = Self::write(value);
        Ok(Self {
            value,
            bytes: bytes_,
        })
    }

    type ValueInput = u128;

    fn from_value(value: Self::ValueInput) -> Result<Self, CodecError> {
        Ok(Self {
            value,
            bytes: Self::write(value),
        })
    }

    fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    type ValueOutput = u128;

    fn get_value(&self) -> Self::ValueOutput {
        self.value
    }
}

// TODO: Find a way to implement Array.
// TODO: It seems we cannot implement the Encodable trait because the from_bytes() function needs
// more than just bytes to deduce what type of information the function has to parse, that is, if I
// properly understood how Array works.
//
// Here is the example where Array has multiple types of data:
// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Login_Success
struct Array {
    /// The `Array` length is known from context when reading certain packets.
    /// Can be positive or zero.
    length: usize,
    types: Vec<DataType>,
    bytes: Vec<u8>,
}

/// Tests mostly written by AI, and not human-checked. 1141

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
            let varint = VarInt::from_bytes(encoded).unwrap();
            let decoded_value = varint.get_value();
            let decoded_length = varint.get_bytes().len();
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
            let varint = VarInt::from_value(*value).unwrap();
            let encoded = varint.get_bytes();
            assert_eq!(encoded, *expected_encoded);
        }
    }

    #[test]
    fn test_varint_roundtrip() {
        let test_values = [
            i32::MIN,
            i32::MIN + 1,
            -1_000_000,
            -1,
            0,
            1,
            1_000_000,
            i32::MAX - 1,
            i32::MAX,
        ];
        for &value in &test_values {
            let varint = VarInt::from_value(value).unwrap();
            let encoded = varint.get_bytes();
            let decoded_varint = VarInt::from_bytes(encoded).unwrap();
            let decoded = decoded_varint.get_value();
            assert_eq!(value, decoded, "Roundtrip failed for value: {}", value);
        }

        let mut rng = rand::thread_rng();
        for _ in 0..10_000 {
            let value = rng.gen::<i32>();
            let varint = VarInt::from_value(value).unwrap();
            let encoded = varint.get_bytes();
            let decoded_varint = VarInt::from_bytes(encoded).unwrap();
            let decoded = decoded_varint.get_value();
            assert_eq!(
                value, decoded,
                "Roundtrip failed for random value: {}",
                value
            );
        }
    }

    #[test]
    fn test_varint_invalid_input() {
        let too_long = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        assert!(matches!(
            VarInt::from_bytes(&too_long),
            Err(CodecError::Decoding(
                DataType::VarInt,
                ErrorReason::ValueTooLarge
            ))
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
            let varlong = VarLong::from_bytes(encoded).unwrap();
            let decoded_value = varlong.get_value();
            let decoded_length = varlong.get_bytes().len();
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
            let varlong = VarLong::from_value(*value).unwrap();
            let encoded = varlong.get_bytes();
            assert_eq!(encoded, *expected_encoded);
        }
    }

    #[test]
    fn test_varlong_roundtrip() {
        let test_values = [
            i64::MIN,
            i64::MIN + 1,
            -1_000_000_000_000,
            -1,
            0,
            1,
            1_000_000_000_000,
            i64::MAX - 1,
            i64::MAX,
        ];

        for &value in &test_values {
            let varlong = VarLong::from_value(value).unwrap();
            let encoded = varlong.get_bytes();
            let decoded_varlong = VarLong::from_bytes(encoded).unwrap();
            let decoded = decoded_varlong.get_value();
            assert_eq!(value, decoded, "Roundtrip failed for value: {}", value);
        }

        let mut rng = rand::thread_rng();
        for _ in 0..10_000 {
            let value = rng.gen::<i64>();
            let varlong = VarLong::from_value(value).unwrap();
            let encoded = varlong.get_bytes();
            let decoded_varlong = VarLong::from_bytes(encoded).unwrap();
            let decoded = decoded_varlong.get_value();
            assert_eq!(
                value, decoded,
                "Roundtrip failed for random value: {}",
                value
            );
        }
    }

    #[test]
    fn test_varlong_invalid_input() {
        let too_long = vec![
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01,
        ];
        assert!(matches!(
            VarLong::from_bytes(&too_long),
            Err(CodecError::Decoding(
                DataType::VarLong,
                ErrorReason::ValueTooLarge
            ))
        ));
    }

    #[test]
    fn test_string_read_valid_ascii() {
        let s = "HELLO";
        let string_bytes = s.as_bytes();
        let length = string_bytes.len();

        let length_varint = VarInt::from_value(length as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let mut data = length_varint;
        data.extend_from_slice(string_bytes);

        let sp = StringProtocol::from_bytes(&data).unwrap();
        assert_eq!(sp.get_value(), s);
    }

    #[test]
    fn test_string_read_valid_utf8() {
        let s = "こんにちは";
        let string_bytes = s.as_bytes();
        let length = string_bytes.len();

        let length_varint = VarInt::from_value(length as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let mut data = length_varint;
        data.extend_from_slice(string_bytes);

        let sp = StringProtocol::from_bytes(&data).unwrap();
        assert_eq!(sp.get_value(), s);
    }

    #[test]
    fn test_string_read_blank_string() {
        let s = "";
        let string_bytes = s.as_bytes();
        let length = string_bytes.len();

        let length_varint = VarInt::from_value(length as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let mut data = length_varint;
        data.extend_from_slice(string_bytes);

        let sp = StringProtocol::from_bytes(&data).unwrap();
        assert!(sp.get_value().is_empty());
    }

    #[test]
    fn test_string_read_too_long_string() {
        let max_allowed_length = 32767;
        let s = "A".repeat(max_allowed_length + 1);
        let string_bytes = s.as_bytes();
        let length = string_bytes.len();

        let length_varint = VarInt::from_value(length as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let mut data = length_varint;
        data.extend_from_slice(string_bytes);

        let string = StringProtocol::from_bytes(&data);
        assert!(matches!(string, Err(_)));
    }

    #[test]
    fn test_string_read_invalid_varint() {
        let invalid_varint = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        let string_bytes = b"HELLO";

        let mut data = invalid_varint;
        data.extend_from_slice(string_bytes);

        match StringProtocol::from_bytes(&data) {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => {
                assert!(matches!(e, CodecError::Decoding(DataType::VarInt, _)));
            }
        }
    }

    #[test]
    fn test_string_read_invalid_utf8() {
        let length = 3;
        let length_varint = VarInt::from_value(length as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let invalid_utf8 = vec![0xFF, 0xFF, 0xFF];

        let mut data = length_varint;
        data.extend_from_slice(&invalid_utf8);

        match StringProtocol::from_bytes(&data) {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => {
                assert!(matches!(
                    e,
                    CodecError::Decoding(DataType::StringProtocol, ErrorReason::InvalidFormat(_))
                ));
            }
        }
    }

    #[test]
    fn test_string_read_incomplete_data() {
        let length = 10;
        let length_varint = VarInt::from_value(length as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let string_bytes = b"HELLO";

        let mut data = length_varint;
        data.extend_from_slice(string_bytes);

        match StringProtocol::from_bytes(&data) {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => {
                assert!(matches!(
                    e,
                    CodecError::Decoding(DataType::StringProtocol, ErrorReason::InvalidFormat(_))
                ));
            }
        }
    }

    #[test]
    fn test_string_read_no_data() {
        let length = 5;
        let data = VarInt::from_value(length as i32)
            .unwrap()
            .get_bytes()
            .to_vec();

        match StringProtocol::from_bytes(&data) {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => {
                assert!(matches!(
                    e,
                    CodecError::Decoding(DataType::StringProtocol, ErrorReason::InvalidFormat(_))
                ));
            }
        }
    }

    #[test]
    fn test_string_read_empty_data() {
        let data: Vec<u8> = Vec::new();

        match StringProtocol::from_bytes(&data) {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => {
                assert!(matches!(
                    e,
                    CodecError::Decoding(DataType::VarInt, ErrorReason::ValueEmpty)
                ));
            }
        }
    }

    #[test]
    fn test_string_read_random_strings() {
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let length = rng.gen_range(1..=100);
            let s: String = (0..length)
                .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
                .collect();
            let string_bytes = s.as_bytes();

            let length_varint = VarInt::from_value(string_bytes.len() as i32)
                .unwrap()
                .get_bytes()
                .to_vec();
            let mut data = length_varint;
            data.extend_from_slice(string_bytes);

            let sp = StringProtocol::from_bytes(&data).unwrap();
            assert_eq!(sp.get_value(), s);
        }
    }

    #[test]
    fn test_write_valid_string() {
        let input = "Hello, World!";
        let varint_bytes = VarInt::from_value(input.len() as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let expected_bytes = [varint_bytes.as_slice(), input.as_bytes()].concat();

        let sp = StringProtocol::from_value(input.to_string()).unwrap();
        assert_eq!(sp.get_bytes(), expected_bytes);
    }

    #[test]
    fn test_write_empty_string() {
        let input = "";
        let varint_bytes = VarInt::from_value(input.len() as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let expected_bytes = varint_bytes;

        let sp = StringProtocol::from_value(input.to_string()).unwrap();
        assert_eq!(sp.get_bytes(), expected_bytes);
    }

    #[test]
    fn test_write_string_exceeding_max_utf16_units() {
        let input: String = std::iter::repeat('𠀋').take(32768).collect();
        match StringProtocol::from_value(input) {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => assert!(matches!(
                e,
                CodecError::Encoding(DataType::StringProtocol, ErrorReason::InvalidFormat(_))
            )),
        }
    }

    #[test]
    fn test_write_string_exceeding_max_data_size() {
        let long_string = "a".repeat(32767 * 3 + 4);
        let string = StringProtocol::from_value(long_string);
        assert!(matches!(string, Err(_)));
    }

    #[test]
    fn test_write_string_with_special_characters() {
        let input = "こんにちは、世界! 🌍";
        let varint_bytes = VarInt::from_value(input.len() as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let expected_bytes = [varint_bytes.as_slice(), input.as_bytes()].concat();

        let sp = StringProtocol::from_value(input.to_string()).unwrap();
        assert_eq!(sp.get_bytes(), expected_bytes);
    }

    #[test]
    fn test_write_string_near_max_length() {
        let max_length = 32767;
        let input: String = std::iter::repeat('a').take(max_length).collect();
        let varint_bytes = VarInt::from_value(input.len() as i32)
            .unwrap()
            .get_bytes()
            .to_vec();
        let expected_bytes = [varint_bytes.as_slice(), input.as_bytes()].concat();

        let sp = StringProtocol::from_value(input.clone()).unwrap();
        assert_eq!(sp.get_bytes(), expected_bytes);
    }

    #[test]
    fn test_write_to_read_loop() {
        let input = "こんにちは、世界! 🌍";
        let sp = StringProtocol::from_value(input.to_string()).unwrap();
        let decoded = StringProtocol::from_bytes(&sp.get_bytes()).unwrap();
        assert_eq!(decoded.get_value(), input);
    }

    #[test]
    fn test_unsigned_short_from_value() {
        let values = [0x0000, 0x0001, 0x00FF, 0x1234, 0xFFFF];

        for &val in &values {
            let us = UnsignedShort::from_value(val).unwrap();
            assert_eq!(us.get_value(), val, "Value mismatch");
            assert_eq!(us.get_bytes(), &val.to_be_bytes(), "Bytes mismatch");
        }
    }

    #[test]
    fn test_unsigned_short_from_bytes_exact() {
        let test_cases = vec![
            (vec![0x00, 0x00], 0x0000),
            (vec![0x00, 0x01], 0x0001),
            (vec![0xAB, 0xCD], 0xABCD),
            (vec![0xFF, 0xFF], 0xFFFF),
        ];

        for (bytes, expected) in test_cases {
            let us = UnsignedShort::from_bytes(&bytes).unwrap();
            assert_eq!(
                us.get_value(),
                expected,
                "Value mismatch for bytes: {:?}",
                bytes
            );
            assert_eq!(
                us.get_bytes(),
                &expected.to_be_bytes(),
                "Bytes mismatch for bytes: {:?}",
                bytes
            );
        }
    }

    #[test]
    fn test_unsigned_short_from_bytes_with_extra_data() {
        let bytes = vec![0x12, 0x34, 0xAB, 0xCD];
        let us = UnsignedShort::from_bytes(&bytes).unwrap();
        assert_eq!(us.get_value(), 0x1234);
        assert_eq!(us.get_bytes(), &0x1234_u16.to_be_bytes());
    }

    #[test]
    fn test_unsigned_short_invalid_input() {
        let bytes = vec![0x12];
        let err = UnsignedShort::from_bytes(&bytes).unwrap_err();
        assert!(matches!(
            err,
            CodecError::Decoding(DataType::UnsignedShort, ErrorReason::ValueTooSmall)
        ));
    }

    #[test]
    fn test_unsigned_short_roundtrip() {
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let value = rng.gen::<u16>();
            let us = UnsignedShort::from_value(value).unwrap();
            let decoded = UnsignedShort::from_bytes(us.get_bytes()).unwrap();
            assert_eq!(
                decoded.get_value(),
                value,
                "Roundtrip failed for value {:#X}",
                value
            );
        }
    }

    // A helper function to generate a sample Uuid value and its bytes.
    fn sample_uuid() -> (u128, [u8; 16]) {
        let value: u128 = 0x1234567890ABCDEF1234567890ABCDEF;
        (value, value.to_be_bytes())
    }

    #[test]
    fn test_uuid_read_valid() {
        let (value, bytes) = sample_uuid();
        let read_result = Uuid::read(&bytes);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), value);
    }

    #[test]
    fn test_uuid_read_invalid_length() {
        let bytes = [0x12u8; 15]; // Not enough bytes
        let read_result = Uuid::read(&bytes);
        assert!(read_result.is_err());
    }

    #[test]
    fn test_uuid_write() {
        let (value, bytes) = sample_uuid();
        let written = Uuid::write(value);
        assert_eq!(written, bytes);
    }

    #[test]
    fn test_uuid_from_bytes() {
        let (value, bytes) = sample_uuid();
        let uuid = Uuid::from_bytes(bytes).unwrap();
        assert_eq!(uuid.value, value);
        assert_eq!(uuid.bytes, bytes);
    }

    #[test]
    fn test_uuid_from_bytes_invalid_length() {
        let bytes = [0x34u8; 8]; // Not enough bytes
        let result = Uuid::from_bytes(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_uuid_from_value() {
        let (value, bytes) = sample_uuid();
        let uuid = Uuid::from_value(value).unwrap();
        assert_eq!(uuid.value, value);
        assert_eq!(uuid.bytes, bytes);
    }

    #[test]
    fn test_uuid_get_bytes() {
        let (value, bytes) = sample_uuid();
        let uuid = Uuid::from_value(value).unwrap();
        assert_eq!(uuid.get_bytes(), &bytes);
    }

    #[test]
    fn test_uuid_get_value() {
        let (value, _) = sample_uuid();
        let uuid = Uuid::from_value(value).unwrap();
        assert_eq!(uuid.get_value(), value);
    }

    #[test]
    fn test_uuid_len() {
        let (value, _) = sample_uuid();
        let uuid = Uuid::from_value(value).unwrap();
        assert_eq!(uuid.len(), 16);
    }

    #[test]
    fn test_uuid_consume_from_bytes() {
        let (value, bytes) = sample_uuid();
        let mut slice: &[u8] = &bytes;
        let uuid = Uuid::consume_from_bytes(&mut slice).unwrap();
        assert_eq!(uuid.value, value);
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn test_uuid_consume_from_bytes_extra_data() {
        let (value, bytes) = sample_uuid();
        let mut input = [0u8; 32];
        input[..16].copy_from_slice(&bytes);
        input[16..].copy_from_slice(&bytes);
        let mut slice: &[u8] = &input;
        let uuid = Uuid::consume_from_bytes(&mut slice).unwrap();
        assert_eq!(uuid.value, value);
        assert_eq!(slice.len(), 16); // 16 bytes consumed
    }

    #[test]
    fn test_uuid_consume_from_bytes_invalid() {
        let bytes = [0x12u8; 15]; // Not enough
        let mut slice: &[u8] = &bytes;
        let result = Uuid::consume_from_bytes(&mut slice);
        assert!(result.is_err());
    }

    // Additional thorough tests:

    #[test]
    fn test_uuid_zero_value() {
        let value: u128 = 0;
        let uuid = Uuid::from_value(value).unwrap();
        assert_eq!(uuid.get_value(), 0);
        assert_eq!(uuid.get_bytes(), &[0u8; 16]);
    }

    #[test]
    fn test_uuid_max_value() {
        let value: u128 = u128::MAX;
        let uuid = Uuid::from_value(value).unwrap();
        assert_eq!(uuid.get_value(), u128::MAX);
        assert_eq!(uuid.get_bytes(), &u128::MAX.to_be_bytes());
    }

    #[test]
    fn test_uuid_round_trip() {
        let (value, _) = sample_uuid();
        let uuid = Uuid::from_value(value).unwrap();
        let round_trip = Uuid::from_bytes(uuid.get_bytes()).unwrap();
        assert_eq!(round_trip.get_value(), value);
        assert_eq!(round_trip.get_bytes(), uuid.get_bytes());
    }

    #[test]
    fn test_uuid_random_values() {
        let values = [
            0x00000000000000000000000000000001u128,
            0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEu128,
            0x00112233445566778899AABBCCDDEEFFu128,
        ];

        for &val in &values {
            let uuid = Uuid::from_value(val).unwrap();
            assert_eq!(uuid.get_value(), val);
            assert_eq!(uuid.get_bytes(), &val.to_be_bytes());
            let from_bytes = Uuid::from_bytes(uuid.get_bytes()).unwrap();
            assert_eq!(from_bytes.get_value(), val);
        }
    }

    #[test]
    fn test_uuid_slice_longer_than_16() {
        let (value, bytes) = sample_uuid();
        let mut long_slice = Vec::from(bytes);
        long_slice.extend_from_slice(&[0xFF; 10]); // extra data
        let uuid = Uuid::from_bytes(&long_slice).unwrap();
        assert_eq!(uuid.get_value(), value);

        let mut slice_ref: &[u8] = &long_slice;
        let consumed_uuid = Uuid::consume_from_bytes(&mut slice_ref).unwrap();
        assert_eq!(consumed_uuid.get_value(), value);
        // Ensure extra bytes remain unconsumed
        assert_eq!(slice_ref.len(), 10);
    }
}
