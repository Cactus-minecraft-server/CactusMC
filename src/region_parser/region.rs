use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};

pub struct Region {
    pub x: i32,
    pub z: i32,
    pub chunks: Vec<Option<Vec<u8>>>,
}

impl Region {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            chunks: vec![None; 1024],
        }
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        Ok(Self::new(0, 0)) 
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        Ok(())
    }
}
