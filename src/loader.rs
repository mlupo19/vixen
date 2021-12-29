use crate::chunk::*;
use glium::Surface;

#[derive(Hash, Eq, PartialEq, Debug)]
struct ChunkCoord {
    x: i32,
    y: i32,
    z: i32,
}

impl ChunkCoord {
    pub fn dx(&self, dx: i32) -> ChunkCoord {
        ChunkCoord {
            x: self.x + dx,
            y: self.y,
            z: self.z,
        }
    }
    pub fn dy(&self, dy: i32) -> ChunkCoord {
        ChunkCoord {
            x: self.x,
            y: self.y + dy,
            z: self.z,
        }
    }
    pub fn dz(&self, dz: i32) -> ChunkCoord {
        ChunkCoord {
            x: self.x,
            y: self.y,
            z: self.z + dz,
        }
    }
}

pub struct ChunkLoader {
    chunk_map: std::collections::HashMap<ChunkCoord, Chunk>,
    generator: crate::terrain::TerrainGenerator,
    load_distance: u16,
    render_distance: u16,
}

impl ChunkLoader {
    pub fn new(seed: u32) -> Self {
        ChunkLoader {
            chunk_map: std::collections::HashMap::new(),
            generator: crate::terrain::TerrainGenerator::new(seed),
            load_distance: 2,
            render_distance: 1,
        }
    }

    pub fn update(&mut self, player: &crate::player::Player, display: &glium::Display) {
        for x in (player.x as i32 / CHUNK_SIZE.0 as i32 - self.load_distance as i32)
            ..(player.x as i32 / CHUNK_SIZE.0 as i32 + self.load_distance as i32)
        {
            for y in (player.y as i32 / CHUNK_SIZE.1 as i32 - self.load_distance as i32)
                ..(player.y as i32 / CHUNK_SIZE.1 as i32 + self.load_distance as i32)
            {
                for z in (player.z as i32 / CHUNK_SIZE.2 as i32 - self.load_distance as i32)
                    ..(player.z as i32 / CHUNK_SIZE.2 as i32 + self.load_distance as i32)
                {
                    let chunk_coord = ChunkCoord { x: x, y: y, z: z };
                    match &self.chunk_map.get(&chunk_coord) {
                        None => {
                            let mut chunk = self.generator.generate_chunk((
                                chunk_coord.x,
                                chunk_coord.y,
                                chunk_coord.z,
                            ));
                            match chunk.get_mesh() {
                                None => {
                                    chunk.gen_mesh(
                                        display,
                                        (
                                            self.chunk_map.get(&chunk_coord.dx(1)),
                                            self.chunk_map.get(&chunk_coord.dx(-1)),
                                            self.chunk_map.get(&chunk_coord.dy(-1)),
                                            self.chunk_map.get(&chunk_coord.dy(1)),
                                            self.chunk_map.get(&chunk_coord.dz(1)),
                                            self.chunk_map.get(&chunk_coord.dz(-1)),
                                        ),
                                    );
                                }
                                Some(_) => (),
                            }
                            self.chunk_map.insert(chunk_coord, chunk);
                        }
                        Some(_) => (),
                    }
                }
            }
        }
    }

    pub fn get_block(&self, (x, y, z): (i32, i32, i32)) -> Option<&Block> {
        let chunk_coord = ChunkCoord {
            x: (x as f32 / CHUNK_SIZE.0 as f32).floor() as i32,
            y: (y as f32 / CHUNK_SIZE.1 as f32).floor() as i32,
            z: (z as f32 / CHUNK_SIZE.2 as f32).floor() as i32,
        };
        self.chunk_map
            .get(&chunk_coord)
            .as_ref()
            .unwrap()
            .get_block((
                (x - chunk_coord.x) as usize,
                (y - chunk_coord.y) as usize,
                (z - chunk_coord.z) as usize,
            ))
    }

    pub fn render<U>(
        &self,
        player: &crate::player::Player,
        target: &mut glium::Frame,
        program: &glium::Program,
        uniforms: &U,
        params: &glium::DrawParameters,
    ) where
        U: glium::uniforms::Uniforms,
    {
        for (coord, chunk) in &self.chunk_map {
            let mesh = chunk.get_mesh();
            match mesh {
                None => (),
                Some(mesh) => match target.draw(
                    mesh,
                    chunk.get_index_buffer().as_ref().unwrap(),
                    program,
                    uniforms,
                    params,
                ) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Error while drawing: {}", e);
                    }
                },
            }
        }
    }
}
