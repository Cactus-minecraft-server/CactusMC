pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub data: Vec<u8>, // Compress data
}

impl Chunk {
    pub fn new(x: i32, z: i32, data: Vec<u8>) -> Self {
        Self { x, z, data }
    }

    pub fn decompress(&self) -> std::io::Result<Vec<u8>> {
        use flate2::read::ZlibDecoder;
        let mut decoder = ZlibDecoder::new(&self.data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    pub fn compress(data: &[u8]) -> std::io::Result<Vec<u8>> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        encoder.finish()
    }
}
