use hematite_nbt::{Blob, Tag};
use std::collections::HashMap;

pub fn create_nbt_blob() -> Blob {
    let mut root = HashMap::new();
    root.insert("Level".to_string(), Tag::Compound(HashMap::new()));
    Blob::new(Some("Chunk".to_string()), Tag::Compound(root))
}

pub fn write_nbt_to_chunk(blob: &Blob) -> std::io::Result<Vec<u8>> {
    let mut data = Vec::new();
    blob.to_writer(&mut data)?;
    Ok(data)
}
