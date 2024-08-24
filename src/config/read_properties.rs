use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, Write};
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

    /// Sets a value for a property key.
    pub fn set_property(&mut self, key: String, value: String) -> Option<String> {
        self.0.insert(key, value)
    }

    pub fn write_without_spaces<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        for (key, value) in &self.0 {
            writeln!(writer, "{}={}", key, value)?;
        }

        Ok(())
    }

    pub fn write_with_spaces<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        for (key, value) in &self.0 {
            writeln!(writer, "{} = {}", key, value)?;
        }

        Ok(())
    }

    pub fn write_aligned<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        let max_length = self.0.keys().map(|k| k.len()).max().unwrap_or_default();

        for (key, value) in &self.0 {
            let padded_key = {
                let pad = " ".repeat(max_length.saturating_sub(key.len()));
                let mut padded = String::with_capacity(max_length);
                padded += key;
                padded += &pad;
                padded
            };

            writeln!(writer, "{} = {}", padded_key, value)?;
        }

        Ok(())
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

pub fn read_properties<R: BufRead>(reader: &mut R) -> Result<Properties, PropertiesParseError> {
    let mut properties = Properties::default();

    for (i, line) in reader.lines().enumerate() {
        let line_number = i + 1;

        let line = line.map_err(|e| PropertiesParseError::new_io(line_number, e))?;

        let line = line.trim(); //remove the spaces or the tabulation
        //add this to avoid panic! when i use # ! for comments
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
