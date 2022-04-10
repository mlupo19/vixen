use crate::chunk::*;
use crate::chunk_mesh::*;
use crate::player;
use crate::texture::*;
use glium::Surface;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem::size_of;
use std::sync::{Arc, RwLock};

/// Consists of the chunk coordinate, vertices, and indices
type MeshData = (ChunkCoord, (Vec<Vertex>, Vec<u16>));
/// Consists of the chunk coordinate, chunk data, and the surrounding chunks
type ChunkWithNeighbors = (ChunkCoord, Arc<RwLock<Chunk>>, NeighborChunks);
/// Used to hold the 6 surrounding chunks of a chunk
type NeighborChunks = [Arc<RwLock<Chunk>>;6];

/// The coordinates of a chunk in chunk space
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        ChunkCoord { x, y, z }
    }
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

/// Responsible for handling of loading and unloading chunks as well as mesh generation/unloading
pub struct ChunkLoader {
    chunk_map: HashMap<ChunkCoord, Arc<RwLock<Chunk>>>,
    mesh_map: HashMap<ChunkCoord, ChunkMesh>,
    queued_chunks: HashSet<ChunkCoord>,
    queued_meshes: HashSet<ChunkCoord>,
    load_distance: u16,
    render_distance: u16,
    simulation_distance: u16,
    chunk_rx: std::sync::mpsc::Receiver<(ChunkCoord, Chunk)>,
    chunk_q: multiqueue::MPMCSender<ChunkCoord>,
    mesh_rx: std::sync::mpsc::Receiver<MeshData>,
    mesh_q: multiqueue::MPMCSender<ChunkWithNeighbors>,

    needs_build: Vec<MeshData>,
    to_generate: Vec<ChunkCoord>,

    texture_map: TextureMap,
}

impl ChunkLoader {
    /// Creates a new chunk loader with world seed
    pub fn new(seed: u32, texture_map: TextureMap) -> Self {
        // Distance from camera that chunks are rendered (number of chunks)
        let render_distance = 10;
        // Distance from camera that chunks are generated/loaded
        let load_distance = render_distance + 1;
        // Distance from camera that AI and physics are updated
        let simulation_distance = 4;

        let generator = crate::terrain::TerrainGenerator::new(seed);

        // Multithreaded queue for sending coordinates of chunks that need to be loaded to worker threads
        let (chunk_q, chunk_q_rec): (
            multiqueue::MPMCSender<ChunkCoord>,
            multiqueue::MPMCReceiver<ChunkCoord>,
        ) = multiqueue::mpmc_queue(
            (load_distance * load_distance * load_distance) as u64 * size_of::<ChunkCoord>() as u64,
        );

        // Multithreaded queue for sending chunk data that need to be loaded to worker threads for building meshes
        let (mesh_q, mesh_q_rec): (
            multiqueue::MPMCSender<ChunkWithNeighbors>,
            multiqueue::MPMCReceiver<ChunkWithNeighbors>,
        ) = multiqueue::mpmc_queue(
            (render_distance * render_distance * render_distance) as u64
                * (size_of::<ChunkCoord>()
                    + size_of::<Arc<RwLock<Chunk>>>()
                    + size_of::<NeighborChunks>()) as u64,
        );

        // Channel for sending loaded chunk back to main thread
        let (chunk_tx, chunk_rx) = std::sync::mpsc::channel();

        // Channel for sending meshes back to main thread
        let (mesh_tx, mesh_rx) = std::sync::mpsc::channel();

        // Threads for loading chunks
        for _ in 0..4 {
            let tx = chunk_tx.clone();
            let chunk_q_rec = chunk_q_rec.clone();
            let generator = generator.clone();

            std::thread::spawn(move || loop {
                // Receive coordinate of chunk to be loaded
                let chunk_coord_res = chunk_q_rec.recv();

                match chunk_coord_res {
                    Ok(chunk_coord) => {
                        // Generate chunk
                        // let chunk = match try_load_from_file(&chunk_coord) {
                        //     None => generator.generate_chunk((chunk_coord.x, chunk_coord.y, chunk_coord.z)),
                        //     Some(chunk) => chunk,
                        // };
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
                    Err(_e) => {
                        break;
                    }
                }
            });
        }

        // Threads for loading meshes
        for _ in 0..4 {
            let tx = mesh_tx.clone();
            let mesh_q_rec = mesh_q_rec.clone();
            let texture_info = texture_map.info.clone();

            std::thread::spawn(move || loop {
                // Receive data for generating mesh
                if let Ok((coord, chunk, neighbors)) = mesh_q_rec.recv() {
                    // Generate mesh data
                    let mesh_data = chunk.read().unwrap().gen_mesh(
                        neighbors,
                        &texture_info,
                    );

                    // Send mesh data to main thread
                    match tx.send((coord, mesh_data)) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error sending mesh data to main thread: {}", e);
                        }
                    }
                };
            });
        }

        ChunkLoader {
            chunk_map: HashMap::new(),
            mesh_map: HashMap::new(),
            queued_chunks: HashSet::new(),
            queued_meshes: HashSet::new(),
            load_distance,
            render_distance,
            simulation_distance,
            chunk_rx,
            chunk_q,
            mesh_rx,
            mesh_q,
            needs_build: Vec::with_capacity(
                (12 * load_distance * load_distance * load_distance) as usize,
            ),
            to_generate: Vec::with_capacity(
                (8 * load_distance * load_distance * load_distance) as usize,
            ),
            texture_map,
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
        while let Ok((coord, chunk)) = self.chunk_rx.try_recv() {
            let chunk = chunk;
            self.chunk_map
                .insert(coord.clone(), Arc::new(RwLock::new(chunk)));
            self.queued_chunks.remove(&coord);
        }

        // Check loaded chunks if they are in render distance and if their meshes are loaded.
        // If not, add them to list of meshes to be generated
        for (coord, chunk) in &mut self.chunk_map {
            if !chunk.read().unwrap().is_empty()
                && in_distance(player, coord, self.render_distance)
            {
                match self.mesh_map.get(coord) {
                    None => {
                        chunk.write().unwrap().request_update();
                        self.to_generate.push(coord.clone());
                    }
                    Some(_) => {
                        if chunk.read().unwrap().needs_update() {
                            self.to_generate.push(coord.clone());
                        }
                    }
                }
            }
        }

        // Find neighbor chunks and send chunk data and neighbors' chunk data to worker thread for mesh building
        for coord in &self.to_generate {
            match self.queued_meshes.get(coord) {
                None => {
                    let neighbors = match get_neighbors(&self.chunk_map, coord) {
                        None => continue,
                        Some(neighbors) => neighbors,
                    };

                    match self.mesh_q.try_send((
                        coord.clone(),
                        self.chunk_map.get(coord).unwrap().clone(),
                        neighbors,
                    )) {
                        Ok(_) => {
                            self.queued_meshes.insert(coord.clone());
                        }
                        Err(e) => {
                            println!(
                                "Error sending chunk data for mesh generation to workers: {}",
                                e
                            );
                        }
                    }
                }
                Some(_) => {}
            }
        }

        // Receive mesh data from worker threads
        while let Ok((coord, mesh_data)) = self.mesh_rx.try_recv() {
            self.queued_meshes.remove(&coord);
            self.needs_build.push((coord.clone(), mesh_data));
        }

        // Build meshes from mesh data and insert them into mesh map
        for (coord, vertices) in &self.needs_build {
            match glium::vertex::VertexBuffer::new(display, &vertices.0[..]) {
                Ok(vb) => {
                    let mesh = ChunkMesh::new(vb, {
                        match glium::IndexBuffer::new(
                            display,
                            glium::index::PrimitiveType::TrianglesList,
                            &vertices.1[..],
                        ) {
                            Ok(buf) => buf,
                            Err(err) => {
                                panic!("Error making index buffer: {}", err);
                            }
                        }
                    });

                    self.mesh_map.insert(coord.clone(), mesh);
                    self.chunk_map
                        .get_mut(coord)
                        .unwrap()
                        .write()
                        .unwrap()
                        .set_updated();
                }
                Err(e) => {
                    println!("Error creating vertex buffer: {:?}", e);
                }
            }
        }

        // Unload meshes out of render distance
        // TODO: Don't drop meshes out of render distance, just don't render them so they don't have to be rebuilt
        // (Be careful of making sure that they are updated if they come back into render distance)
        self.mesh_map
            .retain(|coord, _| in_distance(player, coord, self.render_distance));

        // Unload chunks out of load distance
        self.chunk_map
            .retain(|coord, _| in_distance(player, coord, self.load_distance));

        // Clear temporary lists
        self.needs_build.clear();
        self.to_generate.clear();
    }

    /// Returns block data based on coordinate (world space). Returns none if block is in unloaded chunk
    pub fn get_block(&self, [x, y, z]: [i32; 3]) -> Option<Block> {
        let chunk_coord = ChunkCoord {
            x: (x as f32 / CHUNK_SIZE.0 as f32).floor() as i32,
            y: (y as f32 / CHUNK_SIZE.1 as f32).floor() as i32,
            z: (z as f32 / CHUNK_SIZE.2 as f32).floor() as i32,
        };
        match self.chunk_map.get(&chunk_coord) {
            None => None,
            Some(chunk) => chunk.read().unwrap().get_block((
                (x - chunk_coord.x * CHUNK_SIZE.0 as i32) as usize,
                (y - chunk_coord.y * CHUNK_SIZE.1 as i32) as usize,
                (z - chunk_coord.z * CHUNK_SIZE.2 as i32) as usize,
            )),
        }
    }

    /// Sets block
    pub fn set_block(&self, [x, y, z]: [i32; 3], block: Block) {
        let chunk_coord = ChunkCoord {
            x: (x as f32 / CHUNK_SIZE.0 as f32).floor() as i32,
            y: (y as f32 / CHUNK_SIZE.1 as f32).floor() as i32,
            z: (z as f32 / CHUNK_SIZE.2 as f32).floor() as i32,
        };
        match self.chunk_map.get(&chunk_coord) {
            None => (),
            Some(chunk) => {
                if chunk.write().unwrap().set_block((
                    (x - chunk_coord.x * CHUNK_SIZE.0 as i32) as usize,
                    (y - chunk_coord.y * CHUNK_SIZE.1 as i32) as usize,
                    (z - chunk_coord.z * CHUNK_SIZE.2 as i32) as usize,
                ), block) {
                    if let Some(neighbors) = get_neighbors(&self.chunk_map, &chunk_coord) {
                        neighbors.into_iter().for_each(|n| n.write().unwrap().request_update());
                    }
                }
            },
        }
    }

    /// Returns chunk data based on coordinate (chunk space). Returns none if chunk is not loaded
    pub fn get_chunk(&self, (i, j, k): (i32, i32, i32)) -> Option<Arc<RwLock<Chunk>>> {
        let chunk_coord = ChunkCoord {
            x: (i as f32 / CHUNK_SIZE.0 as f32).floor() as i32,
            y: (j as f32 / CHUNK_SIZE.1 as f32).floor() as i32,
            z: (k as f32 / CHUNK_SIZE.2 as f32).floor() as i32,
        };

        self.chunk_map.get(&chunk_coord).cloned()
    }

    /// Renders chunk meshes
    pub fn render(
        &self,
        target: &mut glium::Frame,
        program: &glium::Program,
        view_projection: [[f32; 4]; 4],
        u_light: [f32; 3],
        params: &glium::DrawParameters,
    ) {
        let frustum = crate::camera::Frustum::new(&view_projection);
        for (chunk_coord, chunk_mesh) in &self.mesh_map {
            if frustum.contains(&[chunk_coord.x, chunk_coord.y, chunk_coord.z]) {
                match target.draw(
                    chunk_mesh.get_mesh(),
                    chunk_mesh.get_indices(),
                    program,
                    &uniform! {
                        view_projection: view_projection,
                        u_light: u_light,
                        diffuse_tex: self.texture_map.base.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest).minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapLinear).anisotropy(32),
                        normal_tex: &self.texture_map.normal,
                        chunk_coords: [(chunk_coord.x as i32 * CHUNK_SIZE.0 as i32) as f32, (chunk_coord.y as i32 * CHUNK_SIZE.1 as i32) as f32, (chunk_coord.z as i32 * CHUNK_SIZE.2 as i32) as f32]
                    },
                    params,
                ) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Error while drawing: {}", e);
                    }
                }
            }
        }
    }

    /// Returns the number of loaded chunks
    pub fn get_number_of_loaded_chunks(&self) -> usize {
        self.chunk_map.len()
    }

    /// Returns the number of loaded meshes
    pub fn get_number_of_loaded_meshes(&self) -> usize {
        self.mesh_map.len()
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

// fn try_load_from_file(chunk_coord: &ChunkCoord) -> Option<Chunk> {
//     let path = format!(
//         "chunks/x{}y{}z{}.chunk",
//         chunk_coord.x, chunk_coord.y, chunk_coord.z
//     );
//     match read_chunk_data_from_file(&path) {
//         None => None,
//         Some(data) => Some(Chunk::from_data(chunk_coord.clone(), data)),
//     }
// }

fn get_neighbors(chunk_map: &HashMap<ChunkCoord, Arc<RwLock<Chunk>>>, coord: &ChunkCoord) -> Option<[Arc<RwLock<Chunk>>;6]> {
    Some([
        match chunk_map.get(&coord.dx(1)) {
            None => return None,
            Some(chunk) => chunk.clone(),
        },
        match chunk_map.get(&coord.dx(-1)) {
            None => return None,
            Some(chunk) => chunk.clone(),
        },
        match chunk_map.get(&coord.dy(-1)) {
            None => return None,
            Some(chunk) => chunk.clone(),
        },
        match chunk_map.get(&coord.dy(1)) {
            None => return None,
            Some(chunk) => chunk.clone(),
        },
        match chunk_map.get(&coord.dz(1)) {
            None => return None,
            Some(chunk) => chunk.clone(),
        },
        match chunk_map.get(&coord.dz(-1)) {
            None => return None,
            Some(chunk) => chunk.clone(),
        },
    ])
}