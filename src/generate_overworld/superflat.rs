// Represents a single chunk section, a 16x16x16 block area
pub struct ChunkSection {
    blocks: [[[u16; 16]; 16]; 16], // Each section is a 16x16x16 block cube
}

// Represents a chunk, which contains multiple chunk sections
pub struct Chunk {
    x: i32,                      // X coordinate of the chunk
    z: i32,                      // Z coordinate of the chunk
    sections: Vec<ChunkSection>, // Collection of chunk sections
}

// Function to generate a chunk at the specified coordinates (x, z)
fn generate_chunk(x: i32, z: i32) -> Chunk {
    let mut sections = Vec::new(); // Initialize a vector to hold chunk sections

    // Generate 4 vertical sections for the chunk
    for section_y in 0..4 {
        let mut blocks = [[[0u16; 16]; 16]; 16]; // Initialize all blocks in this section to 0 (air)

        // Populate the blocks for this section
        for y in 0..16 {
            // Loop over the vertical layer (within this section)
            for z in 0..16 {
                // Loop over the depth
                for x in 0..16 {
                    // Loop over the width
                    blocks[y][z][x] = match section_y {
                        0 => 1,            // Bedrock in the first section
                        1 | 2 => 2,        // Dirt in the second and third sections
                        3 if y == 15 => 3, // Grass on the top layer of the last section
                        _ => 0,            // Air for all other cases
                    };
                }
            }
        }

        // Add the populated section to the sections vector
        sections.push(ChunkSection { blocks });
    }

    // Return the completed chunk
    Chunk { x, z, sections }
}
