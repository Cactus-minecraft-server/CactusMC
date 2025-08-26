use crate::net;
use core::str;
use log::{debug, warn};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use thiserror::Error;
// Remark: dynamic dispatch is what we need in this file, but it would mean (relative to this file)
// heavy refactors for the Encodable trait and maybe the creation of a Parser struct or something.

// TODO: Convert all Vec<u8> into boxed slices of bytes (Box<[u8]>) for performance.
// TODO: Is this TODO foolish, or clever? I mean we don't need to mutate, so a Vec<u8> is useless.

pub trait Encodable: Sized + Default + Debug + Clone + PartialEq + Eq {
    // Note to future-self: we can't make a custom type for `from_bytes` because it contains
    // the parsing logic for each type.

    /// Context for Template types like `Array`.
    type Ctx;

    /// Creates an instance from the first data type from a byte slice.
    /// The input slice remains unmodified.
    fn from_bytes<T: AsRef<[u8]>>(bytes: T, ctx: Self::Ctx) -> Result<Self, CodecError>;

    /// Creates an instance from the first data type from a byte slice.
    /// Read bytes are consumed. For instance, if you were to read an UnsignedByte on [1, 5, 4, 8],
    /// the buffer would then be [4, 8] after the function call.
    fn consume_from_bytes(bytes: &mut &[u8], ctx: Self::Ctx) -> Result<Self, CodecError> {
        let instance = Self::from_bytes(*bytes, ctx)?;
        *bytes = &bytes[instance.size()..];
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

    /// Returns the number of bytes taken by the data type.
    fn size(&self) -> usize {
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
    Array(Vec<DataType>),
    Boolean,
    Optional(Box<DataType>),
    Byte,
    Other(&'static str),
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DataType::VarInt => write!(f, "VarInt"),
            DataType::VarLong => write!(f, "VarLong"),
            DataType::StringProtocol => write!(f, "String"),
            DataType::UnsignedShort => write!(f, "UnsignedShort"),
            DataType::Uuid => write!(f, "UUID"),
            DataType::Array(types) => write!(f, "Array of {types:?}"),
            DataType::Boolean => write!(f, "Boolean"),
            DataType::Optional(optional) => write!(f, "Optional of {:?}", *optional),
            DataType::Byte => write!(f, "Byte"),
            DataType::Other(name) => write!(f, "{}", name),
        }
    }
}

impl From<DataTypeContent> for DataType {
    fn from(value: DataTypeContent) -> Self {
        match value {
            DataTypeContent::VarInt(_) => Self::VarInt,
            DataTypeContent::VarLong(_) => DataType::VarLong,
            DataTypeContent::StringProtocol(_) => DataType::StringProtocol,
            DataTypeContent::UnsignedShort(_) => DataType::UnsignedShort,
            DataTypeContent::Uuid(_) => DataType::Uuid,
            DataTypeContent::Array(array) => {
                let data_types: Vec<DataType> = array
                    .get_value()
                    .iter()
                    .map(|d| (*d).clone().into())
                    .collect();
                DataType::Array(data_types)
            }
            DataTypeContent::Boolean(_) => DataType::Boolean,
            DataTypeContent::Optional(optional) => {
                if let Some(data_type) = optional.get_value() {
                    DataType::Optional(Box::new((*data_type).clone().into()))
                } else {
                    // NOT FINE!
                    // We lose some information here, even though not really.
                    // I'm not clever enough, but, in the end, maybe it is fine?
                    DataType::Optional(Box::new(Self::Other("")))
                }
            }
            DataTypeContent::Byte(_) => DataType::Byte,
            DataTypeContent::Other(_) => DataType::Other(""), // Or however you handle "Other"
        }
    }
}

impl PartialEq<DataTypeContent> for DataType {
    fn eq(&self, other: &DataTypeContent) -> bool {
        other == self // Delegate to the other implementation
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

impl Display for ErrorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
/// Pseudocode of this algorithm taken from https://wiki.vg/Protocol#VarInt_and_VarLong
/// A VarInt may not be longer than 5 bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
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

    /// This function encodes an i32 to a Vec<u8>.
    /// The returned Vec<u8> may not be longer than 5 elements.
    fn write(mut value: i32) -> Result<Vec<u8>, CodecError> {
        let mut result = Vec::<u8>::with_capacity(5);

        loop {
            let byte = (value & Self::SEGMENT_BITS) as u8;

            // Moves the sign bit too by doing bitwise operation on the u32.
            value = ((value as u32) >> 7) as i32;

            // Value == 0 means that it's a positive value, and it's been shifted enough.
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
    type Ctx = ();

    fn from_bytes<T: AsRef<[u8]>>(bytes: T, _: Self::Ctx) -> Result<Self, CodecError> {
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
}

/// Implementation of the LEB128 variable-length compression algorithm.
/// Pseudocode of this algorithm from https://wiki.vg/Protocol#VarInt_and_VarLong.
/// A VarLong may not be longer than 10 bytes.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
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

    /// This function encodes an i64 to a Vec<u8>.
    /// The returned Vec<u8> may not be longer than 10 elements.
    fn write(mut value: i64) -> Result<Vec<u8>, CodecError> {
        let mut result = Vec::<u8>::with_capacity(10);

        loop {
            let byte = (value & Self::SEGMENT_BITS) as u8;

            // Moves the sign bit too by doing bitwise operation on the u32.
            value = ((value as u64) >> 7) as i64;

            // Value == 0 means that it's a positive value, and it's been shifted enough.
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
    type Ctx = ();

    fn from_bytes<T: AsRef<[u8]>>(bytes: T, _: Self::Ctx) -> Result<Self, CodecError> {
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
#[derive(Debug, Default, Clone, Eq, PartialEq)]
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
    /// a UTF-8 string, then convert it to UTF-16 to count the number of code points (also, code
    /// points above U+FFFF count as 2) to check if the String is following or not the rules.
    fn read<T: AsRef<[u8]>>(data: T) -> Result<(String, usize), CodecError> {
        let varint = VarInt::from_bytes(&data, ())?;

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
    type Ctx = ();

    fn from_bytes<T: AsRef<[u8]>>(bytes: T, _: Self::Ctx) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();
        let string: (String, usize) = Self::read(data)?;
        Ok(Self {
            string: string.0,
            // Only take the string, no more.
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
    type Ctx = ();

    fn from_bytes<T: AsRef<[u8]>>(bytes: T, _: Self::Ctx) -> Result<Self, CodecError> {
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Uuid {
    value: u128,
    /// There are 16 bytes in an u128.
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
    /// There are 16 bytes in an u128.
    fn write(value: u128) -> [u8; 16] {
        value.to_be_bytes()
    }
}

impl Encodable for Uuid {
    type Ctx = ();

    fn from_bytes<T: AsRef<[u8]>>(bytes: T, _: Self::Ctx) -> Result<Self, CodecError> {
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

/// Used in `Array` and `Optional` to hold data inside the enum's variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataTypeContent {
    VarInt(VarInt),
    VarLong(VarLong),
    StringProtocol(StringProtocol),
    UnsignedShort(UnsignedShort),
    Uuid(Uuid),
    Array(Array),
    Boolean(Boolean),
    Optional(Box<Optional>),
    Byte(Byte),
    Other(String),
}

impl DataTypeContent {
    /// Returns the length in bytes of the ArrayType.
    pub fn size(&self) -> usize {
        match self {
            DataTypeContent::VarInt(varint) => varint.size(),
            DataTypeContent::VarLong(varlong) => varlong.size(),
            DataTypeContent::StringProtocol(string_protocol) => string_protocol.size(),
            DataTypeContent::UnsignedShort(unsigned_short) => unsigned_short.size(),
            DataTypeContent::Uuid(uuid) => uuid.size(),
            DataTypeContent::Array(array) => array.size(),
            DataTypeContent::Boolean(boolean) => boolean.size(),
            DataTypeContent::Optional(optional) => (*optional).len(),
            DataTypeContent::Byte(v) => v.size(),
            DataTypeContent::Other(_) => 0, // Assuming `Other` doesn't have a meaningful length
        }
    }

    /// Returns the bytes of the ArrayType.
    pub fn get_bytes(&self) -> &[u8] {
        match self {
            DataTypeContent::VarInt(varint) => varint.get_bytes(),
            DataTypeContent::VarLong(varlong) => varlong.get_bytes(),
            DataTypeContent::StringProtocol(string_protocol) => string_protocol.get_bytes(),
            DataTypeContent::UnsignedShort(unsigned_short) => unsigned_short.get_bytes(),
            DataTypeContent::Uuid(uuid) => uuid.get_bytes(),
            DataTypeContent::Array(array) => array.get_bytes(),
            DataTypeContent::Boolean(boolean) => boolean.get_bytes(),
            DataTypeContent::Optional(optional) => (*optional).get_bytes(),
            DataTypeContent::Byte(v) => v.get_bytes(),
            DataTypeContent::Other(_) => &[], // Assuming `Other` doesn't have a meaningful length
        }
    }

    /// Parses a `DataType` from bytes and context, which is a `data_type`.
    pub fn from_bytes<T: AsRef<[u8]>>(
        bytes: T,
        data_type: &DataType,
    ) -> Result<DataTypeContent, CodecError> {
        let mut data = bytes.as_ref();

        match data_type {
            DataType::VarInt => {
                let varint = VarInt::consume_from_bytes(&mut data, ())?;
                Ok(DataTypeContent::VarInt(varint))
            }
            DataType::VarLong => {
                let varlong = VarLong::consume_from_bytes(&mut data, ())?;
                Ok(DataTypeContent::VarLong(varlong))
            }
            DataType::StringProtocol => {
                let string_protocol = StringProtocol::consume_from_bytes(&mut data, ())?;
                Ok(DataTypeContent::StringProtocol(string_protocol))
            }
            DataType::UnsignedShort => {
                let unsigned_short = UnsignedShort::consume_from_bytes(&mut data, ())?;
                Ok(DataTypeContent::UnsignedShort(unsigned_short))
            }
            DataType::Uuid => {
                let uuid = Uuid::consume_from_bytes(&mut data, ())?;
                Ok(DataTypeContent::Uuid(uuid))
            }
            DataType::Array(inner_types) => {
                // Recursive call under the hood.
                //
                // Absolute banger of a line of code (originally)
                let array = Array::consume_from_bytes(&mut data, inner_types.into())?;
                Ok(DataTypeContent::Array(array))
            }
            DataType::Boolean => {
                let boolean = Boolean::consume_from_bytes(&mut data, ())?;
                Ok(DataTypeContent::Boolean(boolean))
            }
            DataType::Optional(inner_type) => {
                let optional = Optional::consume_from_bytes(&mut data, (**inner_type).clone())?;
                Ok(DataTypeContent::Optional(Box::new(optional)))
            }
            DataType::Byte => {
                let byte = Byte::consume_from_bytes(&mut data, ())?;
                Ok(DataTypeContent::Byte(byte))
            }
            DataType::Other(value) => Err(CodecError::Decoding(
                (*data_type).clone(),
                ErrorReason::UnknownValue(format!(
                    "Unexpected 'Other' data type with value: {}",
                    value
                )),
            )),
        }
    }

    /// Parses a data_type from bytes and context, here `data_type` and consumes the bytes buffer.
    pub fn consume_from_bytes(bytes: &mut &[u8], data_type: &DataType) -> Result<Self, CodecError> {
        let instance = Self::from_bytes(&bytes, data_type)?;
        *bytes = &bytes[instance.size()..];
        Ok(instance)
    }
}

impl PartialEq<DataType> for DataTypeContent {
    fn eq(&self, other: &DataType) -> bool {
        match (self, other) {
            (DataTypeContent::VarInt(_), DataType::VarInt) => true,
            (DataTypeContent::VarLong(_), DataType::VarLong) => true,
            (DataTypeContent::StringProtocol(_), DataType::StringProtocol) => true,
            (DataTypeContent::UnsignedShort(_), DataType::UnsignedShort) => true,
            (DataTypeContent::Uuid(_), DataType::Uuid) => true,
            (DataTypeContent::Array(_), DataType::Array(_)) => true, // Matching structure, not content
            (DataTypeContent::Boolean(_), DataType::Boolean) => true,
            (DataTypeContent::Optional(_), DataType::Optional(_)) => true, // Matching structure
            (DataTypeContent::Other(content), DataType::Other(other)) => content == other,
            _ => false, // Fallback for mismatched types
        }
    }
}

/// This impl is for the sake of throwing an error if `DataType` is modified without mirroring
/// changes onto the `ArrayTypes` enum.
///
/// Sadly it's redundant code... But I don't have faith in my memory.
impl DataType {
    pub fn _compiler_happy(&self) -> DataTypeContent {
        match self {
            DataType::VarInt => DataTypeContent::VarInt(VarInt::default()),
            DataType::VarLong => DataTypeContent::VarLong(VarLong::default()),
            DataType::StringProtocol => DataTypeContent::StringProtocol(StringProtocol::default()),
            DataType::UnsignedShort => DataTypeContent::UnsignedShort(UnsignedShort::default()),
            DataType::Uuid => DataTypeContent::Uuid(Uuid::default()),
            DataType::Array(_) => DataTypeContent::Array(Array::default()),
            DataType::Boolean => DataTypeContent::Boolean(Boolean::default()),
            DataType::Optional(_) => DataTypeContent::Optional(Box::new(Optional::default())),
            DataType::Byte => DataTypeContent::Byte(Byte::default()),
            DataType::Other(_) => DataTypeContent::Other(String::new()),
        }
    }
}

/// Quality of Life macro to allow .into() on call sites of the Array and possibly the Optional
/// data types.
/// Macros are white magic, this is awesome
macro_rules! define_array_ctx {
    ($name:ident, $ty:ty) => {
        pub struct $name {
            types: Box<[$ty]>,
        }

        impl $name {
            /// Canonical constructor.
            pub fn new(types: Box<[$ty]>) -> Self {
                Self { types }
            }
        }

        // Our QoL impls
        impl FromIterator<$ty> for $name {
            fn from_iter<T: IntoIterator<Item = $ty>>(iter: T) -> Self {
                Self {
                    types: iter.into_iter().collect(),
                }
            }
        }

        impl From<Vec<$ty>> for $name {
            fn from(value: Vec<$ty>) -> Self {
                Self::new(value.into_boxed_slice())
            }
        }

        impl From<&[$ty]> for $name {
            fn from(slice: &[$ty]) -> Self {
                Self::new(slice.to_vec().into_boxed_slice())
            }
        }

        impl From<&Vec<$ty>> for $name {
            fn from(v: &Vec<$ty>) -> Self {
                Self::new(v.clone().into_boxed_slice())
            }
        }
    };
}

define_array_ctx!(CtxBytes, DataType);
define_array_ctx!(CtxValues, DataTypeContent);

// Here is the example where Array has multiple types of data:
// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol#Login_Success
//
// Docs: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Type:Array
//
/// This represents an Array that can contain multiple types.
pub struct Array {
    /// We don't need to resize or do shenanigans with it, a shared slice is what we need.
    /// Not a Box<T> because with the get_values() method we would have to return an owned value.
    values: Arc<[DataTypeContent]>,
    /// Array dumped to bytes. Basically
    bytes: Arc<[u8]>,
}

impl Array {
    /// Takes an arbitrary-long number of bytes and tries to sequentially parse the `data_types`.
    /// If Ok, returns a tuple: (values, bytes)
    fn read<T>(
        bytes: T,
        data_types: &[DataType],
    ) -> Result<(Arc<[DataTypeContent]>, Arc<[u8]>), CodecError>
    where
        T: AsRef<[u8]>,
    {
        // Mutable to consume it later.
        let mut data: &[u8] = bytes.as_ref();

        // Parse all types into a sliced Arc.
        // Remark: I think this uses an underlying Vec, making the whole process take
        // multiple allocations and maybe a realloc from the Vec to the Arc;
        // not maximally efficient.
        let arr_values: Arc<[DataTypeContent]> = data_types
            .iter()
            .map(|dt| DataTypeContent::consume_from_bytes(&mut data, dt))
            .collect::<Result<_, _>>()?;

        // Number of bytes of the array.
        let total_bytes: usize = arr_values.iter().map(|dt| dt.size()).sum();
        if total_bytes > bytes.as_ref().len() {
            return Err(CodecError::Decoding(
                DataType::Array(data_types.into()),
                ErrorReason::ValueTooLarge,
            ));
        }

        // Put bytes into arr_bytes
        let arr_bytes: &[u8] = &bytes.as_ref()[..total_bytes];

        Ok((arr_values, Arc::from(arr_bytes)))
    }

    // TODO: do passes done. to compute the len and make the bytes and all. Perf issue.
    /// Takes multiple data types and adds all their bytes, sequentially, into an array.
    fn write(values: &[DataTypeContent]) -> Arc<[u8]> {
        // pre-size buffer, then append
        // Single allocation here, no growth.
        // I'm having faith that I correctly implemented .size() for all dts.
        let total: usize = values.iter().map(|dt| dt.size()).sum();
        let mut buf: Vec<u8> = Vec::with_capacity(total);
        for dt in values {
            buf.extend_from_slice(dt.get_bytes());
        }
        buf.into()
    }

    /// Return the number of elements inside the Array.
    /// NOT THE NUMBER OF BYTES; refer to `.size()` to do so.
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl Encodable for Array {
    // TODO rewrite this shitty DataType vs DataTypeContent to be one and only.
    //type Ctx = Box<[DataType]>;
    type Ctx = CtxBytes;

    fn from_bytes<T: AsRef<[u8]>>(bytes: T, ctx: Self::Ctx) -> Result<Self, CodecError> {
        let (v, b) = Self::read(bytes, &ctx.types)?;
        Ok(Self {
            values: v,
            bytes: b,
        })
    }
    type ValueInput = CtxValues;

    fn from_value(value: Self::ValueInput) -> Result<Self, CodecError> {
        let values: Arc<[DataTypeContent]> = value.types.iter().cloned().collect();
        let bytes: Arc<[u8]> = Self::write(&values);
        Ok(Self { values, bytes })
    }

    fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    type ValueOutput = Arc<[DataTypeContent]>;

    /// Returns: `Arc<[DataTypeContent]>`
    fn get_value(&self) -> Self::ValueOutput {
        self.values.clone()
    }
}

impl Default for Array {
    /// Returns an empty Array with no items in it. 0 bytes of length.
    fn default() -> Self {
        Self {
            values: Arc::default(),
            bytes: Arc::default(),
        }
    }
}

/// Names of the data types inside INCLUDING their values.
/// E.g., [VarInt(value), Byte(value)]
/// Pretty mode indents and adds newlines. Making it multiline.
impl Debug for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        warn!("Debug for Array called; not yet implemented");
        if f.alternate() {
            return write!(f, "NOT YET IMPLEMENTED; USE Display");
        }
        write!(f, "NOT YET IMPLEMENTED; USE Display")
    }
}

/// Names of the data types inside WITHOUT the values.
/// E.g., [VarInt, Byte]
/// Pretty mode indents and adds newlines. Making it multiline.
impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fn make_arr_str(
            f: &mut Formatter<'_>,
            dts: &[DataTypeContent],
            is_multiline: bool,
        ) -> fmt::Result {
            let sep: &str = if is_multiline { ",\n" } else { ", " };
            f.write_str("[")?;

            if is_multiline {
                f.write_str("\n")?;
            }

            for (idx, dt) in dts.iter().enumerate() {
                // multiline AND non-last: indent.
                if is_multiline && idx + 1 < dts.len() {
                    f.write_str("\t")?;
                }
                f.write_str(net::utils::name_of(Some(dt)))?;
                // Except last dt.
                if idx + 1 < dts.len() {
                    f.write_str(sep)?;
                }
            }
            f.write_str("]")
        }

        if f.alternate() {
            // println!("{arr:#}");
            make_arr_str(f, self.values.as_ref(), true)
        } else {
            // println!("{arr}");
            make_arr_str(f, self.values.as_ref(), false)
        }
    }
}

impl Clone for Array {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            bytes: self.bytes.clone(),
        }
    }
}
impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values && self.bytes == other.bytes
    }
}
impl Eq for Array {}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Boolean {
    value: bool,
    bytes: [u8; 1],
}

impl Boolean {
    const FALSE_BYTE: u8 = 0x00;
    const TRUE_BYTE: u8 = 0x01;
}

impl Encodable for Boolean {
    type Ctx = ();

    /// Converts a byte slice into a `Boolean` struct.
    ///
    /// # Errors
    /// Returns a `CodecError` if the slice is empty or the first byte is not 0 or 1.
    fn from_bytes<T: AsRef<[u8]>>(bytes: T, _: Self::Ctx) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();

        if data.is_empty() {
            return Err(CodecError::Decoding(
                DataType::Boolean,
                ErrorReason::ValueEmpty,
            ));
        }

        let value = match data[0] {
            Self::FALSE_BYTE => false,
            Self::TRUE_BYTE => true,
            _ => {
                return Err(CodecError::Decoding(
                    DataType::Boolean,
                    ErrorReason::UnknownValue(format!(
                        "Expected either 0 or 1, found: {}",
                        data[0]
                    )),
                ));
            }
        };

        Ok(Self {
            value,
            bytes: [data[0]],
        })
    }

    type ValueInput = bool;

    /// Converts a boolean into a `Boolean` object.
    fn from_value(value: Self::ValueInput) -> Result<Self, CodecError> {
        Ok(Self {
            value,
            bytes: [value as u8],
        })
    }

    /// Returns a reference to the associated byte with the `Boolean` object.
    /// Either 0x00 or 0x01.
    fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    type ValueOutput = bool;

    /// Returns the bool value contained in the `Boolean` object.
    fn get_value(&self) -> Self::ValueOutput {
        self.value
    }
}

/// This data type is alike to `Array`, it's an irregular that does not implement the `Encodable`
/// trait, because we need to have some additional information not found in the bytes buffer to
/// parse and creates it.
///
/// Bytewise, an `Optional` is either 0x00 or the data type it contains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Optional {
    value: Option<DataTypeContent>,
    bytes: Vec<u8>,
}

impl Optional {
    /// Creates an instance from the first data type from a byte slice.
    /// The input slice remains unmodified.
    fn from_bytes<T: AsRef<[u8]>>(bytes: T, data_type: DataType) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();

        if data.is_empty() {
            return Err(CodecError::Encoding(
                DataType::Optional(Box::new(data_type)),
                ErrorReason::ValueTooSmall,
            ));
        }

        // No data type.
        if data[0] == 0x00 {
            return Ok(Self {
                value: None,
                bytes: vec![0x00],
            });
        }

        let value: DataTypeContent = DataTypeContent::from_bytes(data, &data_type)?;
        Ok(Self {
            bytes: data[..value.get_bytes().len()].to_vec(),
            value: Some(value),
        })
    }

    /// Creates an instance from the first data type from a byte slice.
    /// Read bytes are consumed. For instance, if you were to read an UnsignedByte (did I mean an u16?) on [1, 5, 4, 8],
    /// the buffer would then be [4, 8] after the function call.
    ///
    /// TODO: IS THIS CORRECT? SHOULD AN OPTIONAL CONSUME? IF THERE IS NO DATA, THERE SHOULD BE NO CONSUMPTION?
    fn consume_from_bytes(bytes: &mut &[u8], data_type: DataType) -> Result<Self, CodecError> {
        let instance = Self::from_bytes(*bytes, data_type)?;
        *bytes = &bytes[instance.len()..];
        Ok(instance)
    }

    /// Creates an instance from a value.
    fn from_value(value: Option<DataTypeContent>) -> Result<Self, CodecError> {
        if let Some(data_type) = value {
            Ok(Self {
                bytes: data_type.get_bytes().to_vec(),
                value: Some(data_type),
            })
        } else {
            Ok(Self {
                value: None,
                bytes: vec![0x00],
            })
        }
    }

    /// Serializes the instance into bytes
    fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns an Option of a reference to the value represented by this instance.
    fn get_value(&self) -> Option<&DataTypeContent> {
        self.value.as_ref()
    }

    // Returns the length of the encoded data in bytes.
    fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl Default for Optional {
    fn default() -> Self {
        Self {
            value: None,
            bytes: vec![0x00],
        }
    }
}

/// Represents a signed 8-bit integer, two's complement
/// Bounds: An integer between -128 and 127
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Byte {
    value: i8,
    bytes: [u8; 1],
}

impl Byte {
    /// Reads the first byte of the provided data in Big Endian format.
    fn read<T: AsRef<[u8]>>(bytes: T) -> Result<i8, CodecError> {
        let data: &[u8] = bytes.as_ref();

        if data.is_empty() {
            return Err(CodecError::Decoding(
                DataType::Byte,
                ErrorReason::ValueTooSmall,
            ));
        }

        Ok(data[0].cast_signed())
    }

    /// Essentially makes the input byte in Big Endian.
    fn write(value: i8) -> [u8; 1] {
        value.to_be_bytes()
    }
}

impl Encodable for Byte {
    type Ctx = ();

    fn from_bytes<T: AsRef<[u8]>>(bytes: T, _: Self::Ctx) -> Result<Self, CodecError> {
        let data: &[u8] = bytes.as_ref();
        let value: i8 = Self::read(data)?;
        Ok(Self {
            value,
            bytes: [data[0]],
        })
    }

    type ValueInput = i8;

    fn from_value(value: Self::ValueInput) -> Result<Self, CodecError> {
        Ok(Self {
            value,
            bytes: Self::write(value),
        })
    }

    fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    type ValueOutput = i8;

    fn get_value(&self) -> Self::ValueOutput {
        self.value
    }
}

/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Prefixed_Array
/// Represents a Prefixed Array of X:
/// An array prefixed by its length. If the array is empty the length will still be encoded.
///
/// Size: size of VarInt + size of X * length
///
/// Effectively a simple wrapper around Array.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct PrefixedArray {
    value: Array,
    bytes: Vec<u8>,
}
/// Tests written with AI, and not human-checked.
/// Unfortunate, but saves a ton of time.
#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;
    use rand;
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
            let varint = VarInt::from_bytes(encoded, ()).unwrap();
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
            let decoded_varint = VarInt::from_bytes(encoded, ()).unwrap();
            let decoded = decoded_varint.get_value();
            assert_eq!(value, decoded, "Roundtrip failed for value: {}", value);
        }

        let mut rng = rand::rng();
        for _ in 0..10_000 {
            let value = rng.random();
            let varint = VarInt::from_value(value).unwrap();
            let encoded = varint.get_bytes();
            let decoded_varint = VarInt::from_bytes(encoded, ()).unwrap();
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
            VarInt::from_bytes(&too_long, ()),
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
            let varlong = VarLong::from_bytes(encoded, ()).unwrap();
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
            let decoded_varlong = VarLong::from_bytes(encoded, ()).unwrap();
            let decoded = decoded_varlong.get_value();
            assert_eq!(value, decoded, "Roundtrip failed for value: {}", value);
        }

        let mut rng = rand::rng();
        for _ in 0..10_000 {
            let value = rng.random();
            let varlong = VarLong::from_value(value).unwrap();
            let encoded = varlong.get_bytes();
            let decoded_varlong = VarLong::from_bytes(encoded, ()).unwrap();
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
            VarLong::from_bytes(&too_long, ()),
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

        let sp = StringProtocol::from_bytes(&data, ()).unwrap();
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

        let sp = StringProtocol::from_bytes(&data, ()).unwrap();
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

        let sp = StringProtocol::from_bytes(&data, ()).unwrap();
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

        let string = StringProtocol::from_bytes(&data, ());
        assert!(matches!(string, Err(_)));
    }

    #[test]
    fn test_string_read_invalid_varint() {
        let invalid_varint = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        let string_bytes = b"HELLO";

        let mut data = invalid_varint;
        data.extend_from_slice(string_bytes);

        match StringProtocol::from_bytes(&data, ()) {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => {
                assert!(matches!(e, CodecError::Decoding(DataType::VarInt, _)));
            }
        }
    }

    #[test]
    fn test_string_read_invalid_utf8() {
        let length = 3;
        let length_varint = VarInt::from_value(length).unwrap().get_bytes().to_vec();
        let invalid_utf8 = vec![0xFF, 0xFF, 0xFF];

        let mut data = length_varint;
        data.extend_from_slice(&invalid_utf8);

        match StringProtocol::from_bytes(&data, ()) {
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
        let length_varint = VarInt::from_value(length).unwrap().get_bytes().to_vec();
        let string_bytes = b"HELLO";

        let mut data = length_varint;
        data.extend_from_slice(string_bytes);

        match StringProtocol::from_bytes(&data, ()) {
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
        let data = VarInt::from_value(length).unwrap().get_bytes().to_vec();

        match StringProtocol::from_bytes(&data, ()) {
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

        match StringProtocol::from_bytes(&data, ()) {
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
        let mut rng = rand::rng();
        for _ in 0..1000 {
            let s: String = (0..10).map(|_| rng.random::<u8>() as char).collect();
            let string_bytes = s.as_bytes();

            let length_varint = VarInt::from_value(string_bytes.len() as i32)
                .unwrap()
                .get_bytes()
                .to_vec();
            let mut data = length_varint;
            data.extend_from_slice(string_bytes);

            let sp = StringProtocol::from_bytes(&data, ()).unwrap();
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
        let decoded = StringProtocol::from_bytes(&sp.get_bytes(), ()).unwrap();
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
            let us = UnsignedShort::from_bytes(&bytes, ()).unwrap();
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
        let us = UnsignedShort::from_bytes(&bytes, ()).unwrap();
        assert_eq!(us.get_value(), 0x1234);
        assert_eq!(us.get_bytes(), &0x1234_u16.to_be_bytes());
    }

    #[test]
    fn test_unsigned_short_invalid_input() {
        let bytes = vec![0x12];
        let err = UnsignedShort::from_bytes(&bytes, ()).unwrap_err();
        assert!(matches!(
            err,
            CodecError::Decoding(DataType::UnsignedShort, ErrorReason::ValueTooSmall)
        ));
    }

    #[test]
    fn test_unsigned_short_roundtrip() {
        let mut rng = rand::rng();
        for _ in 0..1000 {
            let value = rng.random::<u16>();
            let us = UnsignedShort::from_value(value).unwrap();
            let decoded = UnsignedShort::from_bytes(us.get_bytes(), ()).unwrap();
            assert_eq!(
                decoded.get_value(),
                value,
                "Roundtrip failed for value {:#X}",
                value
            );
        }
    }

    // A helper function to randomerate a sample Uuid value and its bytes.
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
        let uuid = Uuid::from_bytes(bytes, ()).unwrap();
        assert_eq!(uuid.value, value);
        assert_eq!(uuid.bytes, bytes);
    }

    #[test]
    fn test_uuid_from_bytes_invalid_length() {
        let bytes = [0x34u8; 8]; // Not enough bytes
        let result = Uuid::from_bytes(bytes, ());
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
        assert_eq!(uuid.size(), 16);
    }

    #[test]
    fn test_uuid_consume_from_bytes() {
        let (value, bytes) = sample_uuid();
        let mut slice: &[u8] = &bytes;
        let uuid = Uuid::consume_from_bytes(&mut slice, ()).unwrap();
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
        let uuid = Uuid::consume_from_bytes(&mut slice, ()).unwrap();
        assert_eq!(uuid.value, value);
        assert_eq!(slice.len(), 16); // 16 bytes consumed
    }

    #[test]
    fn test_uuid_consume_from_bytes_invalid() {
        let bytes = [0x12u8; 15]; // Not enough
        let mut slice: &[u8] = &bytes;
        let result = Uuid::consume_from_bytes(&mut slice, ());
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
        let round_trip = Uuid::from_bytes(uuid.get_bytes(), ()).unwrap();
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
            let from_bytes = Uuid::from_bytes(uuid.get_bytes(), ()).unwrap();
            assert_eq!(from_bytes.get_value(), val);
        }
    }

    #[test]
    fn test_uuid_slice_longer_than_16() {
        let (value, bytes) = sample_uuid();
        let mut long_slice = Vec::from(bytes);
        long_slice.extend_from_slice(&[0xFF; 10]); // extra data
        let uuid = Uuid::from_bytes(&long_slice, ()).unwrap();
        assert_eq!(uuid.get_value(), value);

        let mut slice_ref: &[u8] = &long_slice;
        let consumed_uuid = Uuid::consume_from_bytes(&mut slice_ref, ()).unwrap();
        assert_eq!(consumed_uuid.get_value(), value);
        // Ensure extra bytes remain unconsumed
        assert_eq!(slice_ref.len(), 10);
    }

    #[test]
    fn test_array_from_bytes_single_varint() {
        // We want to parse a single VarInt (let's pick 12345).
        let varint = VarInt::from_value(12345).unwrap();
        let encoded = varint.get_bytes().to_vec();

        // Our DataType for the array is just [VarInt].
        let data_types = vec![DataType::VarInt];

        // Build the array from bytes.
        let array = Array::from_bytes(&encoded, data_types.into()).unwrap();

        assert_eq!(array.len(), 1);

        // Check length and bytes.
        assert_eq!(
            array.size(),
            encoded.len(),
            "Array length should match encoded VarInt length."
        );
        assert_eq!(
            array.get_bytes(),
            &encoded,
            "Array bytes should match original VarInt bytes."
        );

        // Check that we have exactly one ArrayType::VarInt inside.
        assert_eq!(
            array.get_value().len(),
            1,
            "Should contain exactly one element."
        );
        match &array.get_value()[0] {
            DataTypeContent::VarInt(parsed_varint) => {
                assert_eq!(parsed_varint.get_value(), 12345);
            }
            _ => panic!("Expected first element to be a VarInt."),
        }
    }

    #[test]
    fn test_array_from_bytes_multiple_datatypes() {
        // Build individual Encodables we want in our Array.
        let varint = VarInt::from_value(42).unwrap();
        let varlong = VarLong::from_value(9999999999).unwrap();
        let string = StringProtocol::from_value("Hello".into()).unwrap();
        let ushort = UnsignedShort::from_value(65535).unwrap();
        let uuid_val = Uuid::from_value(0x0123_4567_89AB_CDEF_0123_4567_89AB_CDEF).unwrap();

        // Concatenate their bytes in the same order we list our data types.
        let mut encoded = Vec::new();
        encoded.extend_from_slice(varint.get_bytes());
        encoded.extend_from_slice(varlong.get_bytes());
        encoded.extend_from_slice(string.get_bytes());
        encoded.extend_from_slice(ushort.get_bytes());
        encoded.extend_from_slice(uuid_val.get_bytes());

        // Our DataType array:
        let data_types = vec![
            DataType::VarInt,
            DataType::VarLong,
            DataType::StringProtocol,
            DataType::UnsignedShort,
            DataType::Uuid,
        ];

        // Parse the array from bytes.
        let array = Array::from_bytes(&encoded, (&data_types).into()).unwrap();

        // Confirm length and internal bytes.
        assert_eq!(array.size(), encoded.len());
        assert_eq!(array.get_bytes(), &encoded);
        assert_eq!(array.len(), data_types.len());

        // Check each parsed element in order.
        let parsed = array.get_value();
        assert_eq!(parsed.len(), 5);

        match &parsed[0] {
            DataTypeContent::VarInt(val) => assert_eq!(val.get_value(), 42),
            _ => panic!("First element should be VarInt(42)."),
        }
        match &parsed[1] {
            DataTypeContent::VarLong(val) => assert_eq!(val.get_value(), 9999999999),
            _ => panic!("Second element should be VarLong(9999999999)."),
        }
        match &parsed[2] {
            DataTypeContent::StringProtocol(val) => assert_eq!(val.get_value(), "Hello"),
            _ => panic!("Third element should be StringProtocol(\"Hello\")."),
        }
        match &parsed[3] {
            DataTypeContent::UnsignedShort(val) => assert_eq!(val.get_value(), 65535),
            _ => panic!("Fourth element should be UnsignedShort(65535)."),
        }
        match &parsed[4] {
            DataTypeContent::Uuid(val) => {
                assert_eq!(val.get_value(), 0x0123_4567_89AB_CDEF_0123_4567_89AB_CDEF)
            }
            _ => panic!("Fifth element should be the provided UUID."),
        }
    }

    #[test]
    fn test_array_from_bytes_nested_array() {
        // Build a nested array of one VarInt (e.g., 256).
        let nested_varint = VarInt::from_value(256).unwrap();
        let nested_encoded = nested_varint.get_bytes().to_vec();

        // Outer array has DataType::Array with subtypes [VarInt]
        let nested_data_types = vec![DataType::VarInt];
        let nested_array =
            Array::from_bytes(&nested_encoded, nested_data_types.clone().into()).unwrap();
        // This is our final ArrayType::Array(...) that will be inside the top-level array.
        let array_type_nested = DataTypeContent::Array(nested_array);

        // Build the top-level array from just that single "Array" item.
        // We'll do it with manual byte assembly to show how a nested array might appear in practice.
        let mut top_encoded = Vec::new();
        // The top-level DataType is [Array([...])].
        // According to the code, reading the top-level Array is:
        //   1) read the nested array from the known subtypes.
        top_encoded.extend_from_slice(&nested_encoded);

        // The top-level data types is a single element: DataType::Array([VarInt]).
        let top_data_types = vec![DataType::Array(nested_data_types)];

        // Parse the top-level array.
        let top_array = Array::from_bytes(&top_encoded, top_data_types.into()).unwrap();

        // Confirm we have exactly one element, which is ArrayType::Array(...).
        assert_eq!(top_array.get_value().len(), 1);
        match &top_array.get_value()[0] {
            DataTypeContent::Array(inner) => {
                // The inner array should have exactly one VarInt: 256
                assert_eq!(inner.get_value().len(), 1);
                match &inner.get_value()[0] {
                    DataTypeContent::VarInt(val) => {
                        assert_eq!(val.get_value(), 256);
                    }
                    _ => panic!("Expected VarInt inside nested array."),
                }
            }
            _ => panic!("Expected an ArrayType::Array."),
        }
    }

    #[test]
    fn test_array_from_bytes_empty() {
        // An empty array is possible. Provide zero bytes and zero data types.
        let empty_encoded = Vec::new();
        let empty_data_types = vec![];

        let array = Array::from_bytes(&empty_encoded, empty_data_types.into()).unwrap();
        assert_eq!(array.size(), 0, "Empty array should have length zero.");
        assert!(array.get_bytes().is_empty());
        assert!(array.get_value().is_empty(), "Should have no elements.");
    }

    #[test]
    fn test_array_consume_from_bytes() {
        // Suppose we have multiple data encoded in a single buffer, and we only
        // want to parse the first array. Let's parse a single VarInt array, then
        // see that the leftover data is correct.

        // VarInt(777) plus some trailing bytes (0x01, 0x02).
        let varint = VarInt::from_value(777).unwrap();
        let encoded_varint = varint.get_bytes().to_vec();

        let mut combined = Vec::new();
        combined.extend_from_slice(&encoded_varint);
        combined.push(0x01);
        combined.push(0x02);

        let data_types = vec![DataType::VarInt];

        // We'll parse using consume_from_bytes.
        let mut buffer_slice: &[u8] = &combined;
        let array = Array::consume_from_bytes(&mut buffer_slice, data_types.into()).unwrap();
        let t = (&*array.get_value());

        // The leftover buffer should have 2 bytes (0x01, 0x02).
        assert_eq!(buffer_slice, &[0x01, 0x02]);

        // Check that array is correct.
        assert_eq!(array.size(), encoded_varint.len());
        assert_eq!(array.get_bytes(), &encoded_varint);
        match &array.get_value()[0] {
            DataTypeContent::VarInt(val) => assert_eq!(val.get_value(), 777),
            _ => panic!("Should contain VarInt(777)."),
        }
    }

    #[test]
    fn test_array_from_bytes_error_truncated_input() {
        // Suppose we claim there's a VarInt in the data, but actually pass zero bytes.
        let empty_bytes = Vec::new();
        let data_types = vec![DataType::VarInt];

        let result = Array::from_bytes(&empty_bytes, data_types.into());
        assert!(
            result.is_err(),
            "Should fail with truncated input for VarInt."
        );
        match result.err().unwrap() {
            CodecError::Decoding(dt, reason) => {
                assert_eq!(dt, DataType::VarInt, "Error DataType should be VarInt.");
                assert_eq!(
                    reason,
                    ErrorReason::ValueEmpty,
                    "Should be ValueEmpty error."
                );
            }
            _ => panic!("Expected a decoding error for VarInt with ValueEmpty."),
        }
    }

    #[test]
    fn test_array_from_bytes_error_unknown_other() {
        // Provide a single DataType::Other entry.
        let data_types = vec![DataType::Other("test")];
        let some_bytes = vec![0x01, 0x02, 0x03];

        let result = Array::from_bytes(&some_bytes, data_types.into());
        assert!(result.is_err());
    }

    #[test]
    fn test_array_from_value() {
        // Manually build an Array from a slice of ArrayType.
        let varint_10 = DataTypeContent::VarInt(VarInt::from_value(10).unwrap());
        let str_hello =
            DataTypeContent::StringProtocol(StringProtocol::from_value("Hello".into()).unwrap());
        let my_array_types = vec![varint_10.clone(), str_hello.clone()];

        let array = Array::from_value(my_array_types.into()).unwrap();
        assert_eq!(array.get_value().len(), 2);

        match &array.get_value()[0] {
            DataTypeContent::VarInt(v) => assert_eq!(v.get_value(), 10),
            _ => panic!("First element should be VarInt(10)."),
        }
        match &array.get_value()[1] {
            DataTypeContent::StringProtocol(s) => assert_eq!(s.get_value(), "Hello"),
            _ => panic!("Second element should be StringProtocol(\"Hello\")."),
        }

        // Compare final concatenated bytes with the expected direct concatenation.
        let mut expected_bytes = Vec::new();
        expected_bytes.extend_from_slice(varint_10.get_bytes());
        expected_bytes.extend_from_slice(str_hello.get_bytes());
        assert_eq!(array.get_bytes(), &expected_bytes);
    }

    #[test]
    fn test_from_value_false() {
        let boolean = Boolean::from_value(false).expect("Failed to create Boolean from false");
        assert_eq!(boolean.get_value(), false, "Boolean value should be false");
        assert_eq!(
            boolean.get_bytes(),
            &[Boolean::FALSE_BYTE],
            "Byte representation should be 0x00"
        );
    }

    #[test]
    fn test_from_value_true() {
        let boolean = Boolean::from_value(true).expect("Failed to create Boolean from true");
        assert_eq!(boolean.get_value(), true, "Boolean value should be true");
        assert_eq!(
            boolean.get_bytes(),
            &[Boolean::TRUE_BYTE],
            "Byte representation should be 0x01"
        );
    }

    #[test]
    fn test_from_bytes_false() {
        let bytes = [Boolean::FALSE_BYTE];
        let boolean = Boolean::from_bytes(&bytes, ()).expect("Failed to parse false byte");
        assert_eq!(boolean.get_value(), false, "Parsed value should be false");
        assert_eq!(
            boolean.get_bytes(),
            &[Boolean::FALSE_BYTE],
            "Byte representation should be 0x00"
        );
    }

    #[test]
    fn test_from_bytes_true() {
        let bytes = [Boolean::TRUE_BYTE];
        let boolean = Boolean::from_bytes(&bytes, ()).expect("Failed to parse true byte");
        assert_eq!(boolean.get_value(), true, "Parsed value should be true");
        assert_eq!(
            boolean.get_bytes(),
            &[Boolean::TRUE_BYTE],
            "Byte representation should be 0x01"
        );
    }

    #[test]
    fn test_from_bytes_empty_slice() {
        let bytes: [u8; 0] = [];
        let result = Boolean::from_bytes(&bytes, ());
        assert!(result.is_err(), "Empty slice should result in an error");
        if let Err(CodecError::Decoding(_, reason)) = result {
            match reason {
                ErrorReason::ValueEmpty => (), // expected
                _ => panic!("Expected ErrorReason::ValueEmpty, got {:?}", reason),
            }
        } else {
            panic!("Expected a decoding error");
        }
    }

    #[test]
    fn test_from_bytes_invalid_value() {
        let bytes = [0x05];
        let result = Boolean::from_bytes(&bytes, ());
        assert!(result.is_err(), "Invalid byte should result in an error");
        if let Err(CodecError::Decoding(_, reason)) = result {
            match reason {
                ErrorReason::UnknownValue(msg) => assert!(
                    msg.contains("Expected either 0 or 1, found: 5"),
                    "Error message should indicate unknown byte value"
                ),
                _ => panic!("Expected ErrorReason::UnknownValue, got {:?}", reason),
            }
        } else {
            panic!("Expected a decoding error");
        }
    }

    #[test]
    fn test_consume_from_bytes_true() {
        let mut data: &[u8] = &[Boolean::TRUE_BYTE, 0xFF];
        let boolean = Boolean::consume_from_bytes(&mut data, ()).expect("Failed to consume bytes");
        assert_eq!(boolean.get_value(), true, "Parsed value should be true");
        assert_eq!(data, &[0xFF], "Remaining data should skip the Boolean byte");
    }

    #[test]
    fn test_consume_from_bytes_false() {
        let mut data: &[u8] = &[Boolean::FALSE_BYTE, 0xFF];
        let boolean = Boolean::consume_from_bytes(&mut data, ()).expect("Failed to consume bytes");
        assert_eq!(boolean.get_value(), false, "Parsed value should be false");
        assert_eq!(data, &[0xFF], "Remaining data should skip the Boolean byte");
    }

    #[test]
    fn test_len() {
        let boolean = Boolean::from_value(true).expect("Failed to create Boolean from true");
        assert_eq!(boolean.size(), 1, "Boolean length should be 1 byte");
    }

    #[test]
    fn test_roundtrip_false() {
        let original = Boolean::from_value(false).expect("Failed to create Boolean from false");
        let bytes = original.get_bytes();
        let decoded =
            Boolean::from_bytes(bytes, ()).expect("Failed to decode bytes back into Boolean");
        assert_eq!(
            original, decoded,
            "Roundtrip for false should produce the same object"
        );
    }

    #[test]
    fn test_roundtrip_true() {
        let original = Boolean::from_value(true).expect("Failed to create Boolean from true");
        let bytes = original.get_bytes();
        let decoded =
            Boolean::from_bytes(bytes, ()).expect("Failed to decode bytes back into Boolean");
        assert_eq!(
            original, decoded,
            "Roundtrip for true should produce the same object"
        );
    }

    #[test]
    fn test_optional_default() {
        let opt = Optional::default();
        assert_eq!(
            opt.get_value(),
            None,
            "Default Optional should have None value"
        );
        assert_eq!(opt.get_bytes(), &[0x00], "Default bytes should be [0x00]");
        assert_eq!(opt.len(), 1, "Length should be 1 for the default Optional");
    }

    #[test]
    fn test_optional_from_value_none() {
        let opt = Optional::from_value(None).expect("Failed to create Optional from None");
        assert_eq!(opt.get_value(), None, "Optional should store None");
        assert_eq!(
            opt.get_bytes(),
            &[0x00],
            "Optional(None) should serialize to [0x00]"
        );
        assert_eq!(opt.len(), 1, "Optional(None) has length 1");
    }

    #[test]
    fn test_optional_from_value_some_boolean_true() {
        // Create a Boolean(true) as DataTypeContent
        let boolean_true = DataTypeContent::Boolean(Boolean {
            value: true,
            bytes: [0x01],
        });
        let opt = Optional::from_value(Some(boolean_true.clone()))
            .expect("Failed to create Optional from Some(Boolean(true))");

        assert_eq!(
            opt.get_value(),
            Some(&boolean_true),
            "Optional should store Boolean(true)"
        );
        assert_eq!(
            opt.get_bytes(),
            &[0x01],
            "Bytes should be [0x01] for Boolean(true)"
        );
        assert_eq!(opt.len(), 1, "Boolean(true) is 1 byte in length");
    }

    #[test]
    fn test_optional_from_value_some_boolean_false() {
        let boolean_false = DataTypeContent::Boolean(Boolean {
            value: false,
            bytes: [0x00],
        });
        // Note: from_value(Some(...)) with a false-Boolean yields the same byte [0x00]
        // as an empty Optional. This may be acceptable or might need distinct handling
        // in your protocol logic.
        let opt = Optional::from_value(Some(boolean_false.clone()))
            .expect("Failed to create Optional from Some(Boolean(false))");

        assert_eq!(
            opt.get_value(),
            Some(&boolean_false),
            "Optional should store Boolean(false)"
        );
        assert_eq!(
            opt.get_bytes(),
            &[0x00],
            "Bytes [0x00] also indicates None, watch for collisions"
        );
        assert_eq!(opt.len(), 1, "Boolean(false) is 1 byte in length");
    }

    #[test]
    fn test_optional_from_bytes_empty_slice() {
        let data: [u8; 0] = [];
        let result = Optional::from_bytes(&data, DataType::Boolean);
        assert!(result.is_err(), "Empty slice should yield an error");
        if let Err(CodecError::Encoding(dt, reason)) = result {
            assert_eq!(dt, DataType::Optional(Box::new(DataType::Boolean)));
            match reason {
                ErrorReason::ValueTooSmall => {} // expected
                _ => panic!("Expected ErrorReason::ValueTooSmall, got {:?}", reason),
            }
        } else {
            panic!("Expected Encoding error due to ValueTooSmall");
        }
    }

    #[test]
    fn test_optional_from_bytes_none() {
        let data = [0x00];
        let opt =
            Optional::from_bytes(&data, DataType::Boolean).expect("Failed to parse None variant");
        assert_eq!(opt.get_value(), None, "Should parse None from [0x00]");
        assert_eq!(opt.get_bytes(), &[0x00], "Bytes should be [0x00]");
        assert_eq!(opt.len(), 1, "Length should be 1 for None");
    }

    #[test]
    fn test_optional_from_bytes_some_boolean_true() {
        // 0x01 for Boolean(true)
        let data = [0x01];
        let opt = Optional::from_bytes(&data, DataType::Boolean)
            .expect("Failed to parse Optional(Boolean(true)) from bytes");
        let inner_value = opt.get_value().expect("Should be Some(Boolean(true))");
        match inner_value {
            DataTypeContent::Boolean(b) => assert!(
                b.value,
                "Boolean inside Optional should be 'true' for [0x01]"
            ),
            _ => panic!("Expected Boolean variant"),
        }
        assert_eq!(opt.get_bytes(), &[0x01], "Bytes should be [0x01]");
        assert_eq!(opt.len(), 1, "Length of Boolean(true) is 1 byte");
    }

    #[test]
    fn test_optional_consume_from_bytes_none() {
        let mut data: &[u8] = &[0x00, 0xFF];
        let opt = Optional::consume_from_bytes(&mut data, DataType::Boolean)
            .expect("Should consume None (0x00) from the bytes");
        assert_eq!(opt.get_value(), None, "Optional should be None");
        assert_eq!(data, &[0xFF], "Should have consumed 1 byte of data");
    }

    #[test]
    fn test_optional_consume_from_bytes_some_boolean_true() {
        let mut data: &[u8] = &[0x01, 0xAB];
        let opt = Optional::consume_from_bytes(&mut data, DataType::Boolean)
            .expect("Should consume Some(Boolean(true)) from the bytes");
        let val = opt.get_value().expect("Expected Some(Boolean(true))");
        match val {
            DataTypeContent::Boolean(b) => {
                assert!(b.value, "Should parse Boolean(true) for [0x01]")
            }
            _ => panic!("Expected Boolean variant"),
        }
        assert_eq!(
            data,
            &[0xAB],
            "One byte consumed, remainder should be [0xAB]"
        );
    }

    #[test]
    fn test_optional_roundtrip_none() {
        let original = Optional::from_value(None).expect("Creation of Optional(None) failed");
        let serialized = original.get_bytes().to_vec();
        let deserialized = Optional::from_bytes(&serialized, DataType::Boolean)
            .expect("Deserialization of Optional(None) failed");
        assert_eq!(
            deserialized, original,
            "Round-trip None should match the original"
        );
    }

    #[test]
    fn test_optional_roundtrip_some_boolean_true() {
        let boolean_true = DataTypeContent::Boolean(Boolean {
            value: true,
            bytes: [0x01],
        });
        let original = Optional::from_value(Some(boolean_true))
            .expect("Creation of Optional(Boolean(true)) failed");
        let serialized = original.get_bytes().to_vec();
        let deserialized = Optional::from_bytes(&serialized, DataType::Boolean)
            .expect("Deserialization of Optional(Boolean(true)) failed");
        assert_eq!(
            deserialized, original,
            "Round-trip Some(Boolean(true)) should match the original"
        );
    }

    #[test]
    fn test_optional_invalid_boolean_byte() {
        // Non-zero, non-0x01 byte (e.g., 0x05) should fail for Boolean decoding
        let data = [0x05];
        let result = Optional::from_bytes(&data, DataType::Boolean);
        assert!(result.is_err(), "Should fail for an invalid Boolean byte");
        if let Err(CodecError::Decoding(dt, reason)) = result {
            assert_eq!(dt, DataType::Boolean, "Decoding error for invalid Boolean");
            match reason {
                ErrorReason::UnknownValue(msg) => {
                    assert!(
                        msg.contains("Expected either 0 or 1, found: 5"),
                        "Error message should clarify the invalid byte"
                    )
                }
                _ => panic!("Expected ErrorReason::UnknownValue, got {:?}", reason),
            }
        } else {
            panic!("Expected a decoding error for the invalid byte");
        }
    }

    // ----- Byte data type -----

    #[test]
    fn test_byte_all_valid_values() {
        for v in i8::MIN..=i8::MAX {
            // Test from values
            let b_val = Byte::from_value(v).unwrap();
            assert_eq!(b_val.get_value(), v);
            assert_eq!(b_val.get_bytes(), v.to_be_bytes());
            assert_eq!(b_val.size(), 1);

            // Test from bytes
            let b_bytes = Byte::from_bytes(v.to_be_bytes(), ()).unwrap();
            assert_eq!(b_bytes.get_value(), v);
            assert_eq!(b_bytes.get_bytes(), v.to_be_bytes());
            assert_eq!(b_bytes.size(), 1);
        }

        for v in i8::MIN..=i8::MAX {
            let b = Byte::from_value(v).unwrap();
            assert_eq!(Byte::from_bytes(b.get_bytes(), ()).unwrap(), b);
        }
    }

    #[test]
    fn test_byte_all_binary_values() {
        for v in u8::MIN..=u8::MAX {
            let b = Byte::from_bytes(v.to_be_bytes(), ()).unwrap();

            assert_eq!(b.get_value(), v.cast_signed());
            assert_eq!(b.get_bytes(), v.to_be_bytes());
            assert_eq!(b.size(), 1);
        }
    }

    #[test]
    fn byte_from_bytes_rejects_wrong_lengths() {
        assert!(Byte::from_bytes([], ()).is_err());
        assert!(Byte::from_bytes([0, 1], ()).is_ok());
    }
}
