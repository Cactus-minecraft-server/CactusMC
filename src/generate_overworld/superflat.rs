struct ChunkSection {
    blocks: [[[u16; 16]; 16]; 4], // 4 for the 4 layers, 16 for the chunk border, u16 for the blocks positions
}
pub struct Chunck {
    x: i32,
    z: i32,
    sections: Vec<ChunkSection>,
}

fn generate_chunk(x: i32, z: i32, radius: u16) -> Chunck {
    let mut blocks = [[[0u16; 16]; 16]; 4];

    for y in 0..4 {
        //height
        for z in 0..16 {
            // widht
            for x in 0..16 {
                //widht
                blocks[y][z][x] = match y {
                    0 => 1,     //bedrock
                    1 | 2 => 2, //dirt 1 | 2 -> layers 1 and 2
                    3 => 3,     //grass
                    _ => 0,     //air
                };
            }
        }
    }

    todo!();

    //Chunck {
    //    x,
    //    z,
    //    sections: blocks,
    //}
}