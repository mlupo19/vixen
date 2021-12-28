// Terrain Generation

use noise::NoiseFn;
use noise::Seedable;

use crate::chunk::*;

pub struct TerrainGenerator {
    seed: u32,
    noise: noise::Perlin
}

impl TerrainGenerator {

    pub fn new(seed: u32) -> TerrainGenerator {
        let mut noise = noise::Perlin::new();
        noise.set_seed(seed);
        TerrainGenerator {seed: seed, noise:noise}
    }

    pub fn generate_chunk(&self, (x,y,z): (i32, i32, i32)) -> Chunk {
        let mut out = Chunk::empty((x,y,z));
        
        let mut heights = ndarray::Array2::<u8>::zeros((CHUNK_SIZE.0, CHUNK_SIZE.2));
        for i in 0..CHUNK_SIZE.0 {
            for j in 0..CHUNK_SIZE.2 {
                println!("{}", ((self.noise.get([5.0 / (i as f64 + 0.5), 5.0 / (j as f64 + 0.5)]) + 3.0) * 5.0));
                let height = ((self.noise.get([5.0 / (i as f64 + 0.5), 5.0 / (j as f64 + 0.5)]) + 3.0) * 5.0);
                heights[(i,j)] = height as u8;
            }
        }

        for i in 0..CHUNK_SIZE.0 {
            for j in 0..CHUNK_SIZE.1 {
                for k in 0..CHUNK_SIZE.2 {
                    if j as u8 <= heights[(i,k)] {
                        out.set_block((i,j,k), Block::new(1, 5.0));
                    }
                }
            }
        }
        
        out
    }

}