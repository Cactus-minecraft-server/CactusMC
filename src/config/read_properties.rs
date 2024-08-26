use std::collections::HashMap;
use std::error::Error;
use std::io::BufRead;
use std::{fmt, io};

#[derive(Debug)]
pub struct PropertyNotFoundError<'a>(&'a str);

impl<'a> Error for PropertyNotFoundError<'a> {}

impl<'a> fmt::Display for PropertyNotFoundError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Property {:?} not found", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Properties(HashMap<String, String>);

impl From<HashMap<String, String>> for Properties {
    fn from(value: HashMap<String, String>) -> Self {
        Self(value)
    }
}

impl Properties {
    /// Gets the corresponding value for a property key.
    pub fn get_property<'a>(&self, key: &'a str) -> Result<&'_ str, PropertyNotFoundError<'a>> {
        self.0
            .get(key)
            .map(String::as_ref)
            .ok_or(PropertyNotFoundError(key))
    }
}

#[derive(Debug)]
pub struct PropertiesParseError {
    pub line_number: usize,
    pub kind: PropertiesParseErrorKind,
}

impl PropertiesParseError {
    fn new_io(line_number: usize, error: io::Error) -> Self {
        Self {
            line_number,
            kind: PropertiesParseErrorKind::Io(error),
        }
    }

    fn new_invalid_kvp(line_number: usize, line: &str) -> Self {
        Self {
            line_number,
            kind: PropertiesParseErrorKind::InvalidKeyValuePair(InvalidKeyValuePairError(
                line.to_string(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct InvalidKeyValuePairError(String);

impl Error for InvalidKeyValuePairError {}

impl fmt::Display for InvalidKeyValuePairError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot parse a key-value pair from {:?}", self.0)
    }
}

#[derive(Debug)]
pub enum PropertiesParseErrorKind {
    Io(io::Error),
    InvalidKeyValuePair(InvalidKeyValuePairError),
}

impl Error for PropertiesParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            PropertiesParseErrorKind::Io(e) => Some(e),
            PropertiesParseErrorKind::InvalidKeyValuePair(e) => Some(e),
        }
    }
}

impl fmt::Display for PropertiesParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error while parsing properties file (line {})",
            self.line_number
        )
    }
}

/// Parses a configuration file
///
/// # Arguments
/// * `reader` - A buffered reader that provides the input data.
///
/// # Returns
///
/// Returns a Result of `Properties`
pub fn read_properties<R: BufRead>(reader: &mut R) -> Result<Properties, PropertiesParseError> {
    let mut properties = Properties::default();

    for (i, line) in reader.lines().enumerate() {
        let line_number = i + 1;

        let line = line.map_err(|e| PropertiesParseError::new_io(line_number, e))?;

        let line = line.trim(); // Remove the spaces or tabulations

        // This check allows commenting with either "#" or "!".
        if line.is_empty() || line.starts_with("#") || line.starts_with("!") {
            continue;
        }

        let (field, value) = line
            .split_once('=')
            .ok_or_else(|| PropertiesParseError::new_invalid_kvp(line_number, &line))?;

        let key = field.trim().to_string();
        let value = value.trim().to_string();
        properties.0.insert(key, value);
    }

    Ok(properties)
}
