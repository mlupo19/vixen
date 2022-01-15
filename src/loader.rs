use crate::chunk::*;
use crate::chunk_mesh::*;
use crate::player;
use glium::Surface;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
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
    mesh_map: std::collections::HashMap<ChunkCoord, ChunkMesh>,
    queued_chunks: std::collections::HashSet<ChunkCoord>,
    load_distance: u16,
    render_distance: u16,
    simulation_distance: u16,
    rx: std::sync::mpsc::Receiver<(ChunkCoord, Chunk)>,
    chunk_q: multiqueue::MPMCSender<ChunkCoord>,
    pool: scoped_threadpool::Pool,

    updated_chunks: Vec<ChunkCoord>,
    needs_build: Vec<(ChunkCoord, (Vec<Vertex>, Vec<u16>))>,
    to_generate: Vec<ChunkCoord>,
}

impl ChunkLoader {

    /// Creates a new chunk loader with world seed
    pub fn new(seed: u32) -> Self {
        // Distances
        let render_distance = 4;
        let load_distance = render_distance + 1;
        let simulation_distance = 4;

        let generator = crate::terrain::TerrainGenerator::new(seed);

        // Multithreaded queue for sending coordinates of chunks that need to be loaded to worker threads
        let (chunk_q, chunk_q_rec): (
            multiqueue::MPMCSender<ChunkCoord>,
            multiqueue::MPMCReceiver<ChunkCoord>,
        ) = multiqueue::mpmc_queue(
            (load_distance * load_distance * load_distance) as u64 * 12 * 100,
        );

        // Channel for sending loaded chunk back to main thread
        let (tx, rx) = std::sync::mpsc::channel();

        // Threads for loading chunks
        for _ in 0..7 {
            let tx = tx.clone();
            let chunk_q_rec = chunk_q_rec.clone();
            let generator = generator.clone();

            std::thread::spawn(move || loop {
                // Receive coordinate of chunk to be loaded
                let chunk_coord_res = chunk_q_rec.recv();

                match chunk_coord_res {
                    Ok(chunk_coord) => {
                        // Generate chunk
                        let chunk =
                            generator.generate_chunk((chunk_coord.x, chunk_coord.y, chunk_coord.z));

                        // Send generated chunk back to main thread
                        match tx.send((chunk_coord, chunk)) {
                            Ok(_) => (),
                            Err(e) => {
                                println!("Error sending chunk to main thread: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error receiving chunk coord from main thread: {}", e);
                    }
                }
            });
        }

        ChunkLoader {
            chunk_map: std::collections::HashMap::new(),
            mesh_map: std::collections::HashMap::new(),
            queued_chunks: std::collections::HashSet::new(),
            load_distance,
            render_distance,
            simulation_distance,
            rx,
            chunk_q,
            pool: scoped_threadpool::Pool::new(7),

            updated_chunks: Vec::with_capacity(
                (12 * load_distance * load_distance * load_distance) as usize,
            ),
            needs_build: Vec::with_capacity(
                (12 * load_distance * load_distance * load_distance) as usize,
            ),
            to_generate: Vec::with_capacity(
                (8 * load_distance * load_distance * load_distance) as usize,
            ),
        }
    }

    /// Queues the chunks that need to be loaded based on player position, then
    /// inserts the next loaded chunk and generates meshes for chunks within render distance.
    /// To be called on the main thread once per game tick.
    pub fn update(&mut self, player: &crate::player::Player, display: &glium::Display) {

        // Check chunks surrounding player and queue them to be loaded if not already
        for x in (player.x as i32 / CHUNK_SIZE.0 as i32 - self.load_distance as i32)
            ..(player.x as i32 / CHUNK_SIZE.0 as i32 + self.load_distance as i32)
        {
            for y in (player.y as i32 / CHUNK_SIZE.1 as i32 - self.load_distance as i32)
                ..(player.y as i32 / CHUNK_SIZE.1 as i32 + self.load_distance as i32)
            {
                for z in (player.z as i32 / CHUNK_SIZE.2 as i32 - self.load_distance as i32)
                    ..(player.z as i32 / CHUNK_SIZE.2 as i32 + self.load_distance as i32)
                {
                    let chunk_coord = ChunkCoord { x, y, z };
                    let mut to_update = false;

                    match &self.chunk_map.get(&chunk_coord) {
                        None => match self.queued_chunks.get(&chunk_coord) {
                            // Queue chunk to be loaded
                            None => match self.chunk_q.try_send(chunk_coord.clone()) {
                                Ok(_) => {
                                    to_update = true;
                                }
                                Err(e) => {
                                    println!("Error sending chunk coord to workers: {}", e);
                                }
                            },
                            Some(_) => (),
                        },
                        Some(_chunk) => {}
                    }
                    if to_update {
                        self.queued_chunks.insert(chunk_coord);
                    }
                }
            }
        }

        // Receive loaded chunk from worker
        while let Ok((coord, chunk)) = self.rx.try_recv() {
            let chunk = chunk;
            self.chunk_map.insert(coord.clone(), chunk);
            self.queued_chunks.remove(&coord);
        }

        // Check loaded chunks if they are in render distance and if their meshes are loaded.
        // If not, add them to list of meshes to be generated
        for (coord, chunk) in &mut self.chunk_map {
            if !chunk.is_empty() && in_distance(&player, coord, self.render_distance) {
                match self.mesh_map.get(&coord) {
                    None => {
                        chunk.request_update();
                        self.to_generate.push(coord.clone());
                    }
                    Some(_) => {
                        if chunk.needs_update() {
                            self.to_generate.push(coord.clone());
                        }
                    }
                }
            }
        }

        // Use scoped threadpool to generate mesh data
        self.pool.scoped(|scope| {
            scope.execute(|| {
                for coord in &self.to_generate {
                    let neighbors = (
                        self.chunk_map.get(&coord.dx(1)),
                        self.chunk_map.get(&coord.dx(-1)),
                        self.chunk_map.get(&coord.dy(-1)),
                        self.chunk_map.get(&coord.dy(1)),
                        self.chunk_map.get(&coord.dz(1)),
                        self.chunk_map.get(&coord.dz(-1)),
                    );

                    if let Some(vertices) = self.chunk_map.get(&coord).unwrap().gen_mesh(neighbors)
                    {
                        self.needs_build.push((coord.clone(), vertices));
                    }
                }
            });
        });

        // Build meshes from mesh data and insert them into mesh map
        for (coord, vertices) in &self.needs_build {
            match glium::vertex::VertexBuffer::new(display, &vertices.0[..]) {
                Ok(vb) => {
                    let mesh = ChunkMesh::new(Some(vb), {
                        match glium::IndexBuffer::new(
                            display,
                            glium::index::PrimitiveType::TrianglesList,
                            &vertices.1[..],
                        ) {
                            Ok(buf) => Some(buf),
                            Err(err) => {
                                println!("Error making index buffer: {}", err);
                                None
                            }
                        }
                    });

                    self.mesh_map.insert(coord.clone(), mesh);
                    self.updated_chunks.push(coord.clone());
                }
                Err(e) => {
                    println!("Error creating vertex buffer: {:?}", e);
                }
            }
        }

        // Mark all chunks that needed their meshes built/rebuilt as updated
        for chunk_coord in &self.updated_chunks {
            self.chunk_map.get_mut(&chunk_coord).unwrap().set_updated();
        }

        // Unload meshes out of render distance
        // TODO: Don't drop meshes out of render distance, just don't render them so they don't have to be rebuilt
        // (Be careful of making sure that they are updated if they come back into render distance)
        self.mesh_map.retain(|coord, _| {
            in_distance(&player, &coord, self.render_distance)
        });

        // Unload chunks out of load distance
        self.chunk_map.retain(|coord, _| {
            in_distance(&player, &coord, self.load_distance)
        });

        // Clear temporary lists
        self.updated_chunks.clear();
        self.needs_build.clear();
        self.to_generate.clear();
    }

    /// Returns block data based on coordinate (world space). Returns none if block is in unloaded chunk
    pub fn get_block(&self, [x, y, z]: [i32;3]) -> Option<&Block> {
        let chunk_coord = ChunkCoord {
            x: (x as f32 / CHUNK_SIZE.0 as f32).floor() as i32,
            y: (y as f32 / CHUNK_SIZE.1 as f32).floor() as i32,
            z: (z as f32 / CHUNK_SIZE.2 as f32).floor() as i32,
        };
        match self.chunk_map.get(&chunk_coord).as_ref() {
            None => None,
            Some(block) => block.get_block((
                (x - chunk_coord.x * CHUNK_SIZE.0 as i32) as usize,
                (y - chunk_coord.y * CHUNK_SIZE.1 as i32) as usize,
                (z - chunk_coord.z * CHUNK_SIZE.2 as i32) as usize,
            )),
        }
    }

    /// Returns chunk data based on coordinate (chunk space). Returns none if chunk is not loaded
    pub fn get_chunk(&self, (i, j, k): (i32, i32, i32)) -> Option<&Chunk> {
        let chunk_coord = ChunkCoord {
            x: (i as f32 / CHUNK_SIZE.0 as f32).floor() as i32,
            y: (j as f32 / CHUNK_SIZE.1 as f32).floor() as i32,
            z: (k as f32 / CHUNK_SIZE.2 as f32).floor() as i32,
        };

        self.chunk_map.get(&chunk_coord)
    }

    /// Renders chunk meshes
    pub fn render(
        &self,
        target: &mut glium::Frame,
        program: &glium::Program,
        view: [[f32; 4]; 4],
        perspective: [[f32; 4]; 4],
        u_light: [f32; 3],
        diffuse_tex: &glium::texture::SrgbTexture2d,
        normal_tex: &glium::texture::Texture2d,
        params: &glium::DrawParameters,
    ) {
        for (chunk_coord, chunk_mesh) in &self.mesh_map {
            let mesh = chunk_mesh.get_mesh();
            match mesh {
                None => (),
                Some(mesh) => match target.draw(
                    mesh,
                    chunk_mesh.get_indices().as_ref().unwrap(),
                    program,
                    &uniform! {view: view, perspective: perspective, u_light: u_light, diffuse_tex: diffuse_tex, normal_tex: normal_tex, chunk_coords: [(chunk_coord.x as i32 * CHUNK_SIZE.0 as i32) as f32, (chunk_coord.y as i32 * CHUNK_SIZE.1 as i32) as f32, (chunk_coord.z as i32 * CHUNK_SIZE.2 as i32) as f32]},
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

#[inline]
fn in_distance(player: &player::Player, coord: &ChunkCoord, distance: u16) -> bool {
    ((player.x as i32 - (coord.x * CHUNK_SIZE.0 as i32)) / CHUNK_SIZE.0 as i32).abs()
                <= distance as i32
                && ((player.y as i32 - (coord.y * CHUNK_SIZE.1 as i32)) / CHUNK_SIZE.1 as i32).abs()
                    <= distance as i32
                && ((player.z as i32 - (coord.z * CHUNK_SIZE.2 as i32)) / CHUNK_SIZE.2 as i32).abs()
                    <= distance as i32
}