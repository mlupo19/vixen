use crate::chunk_mesh::*;

pub const CHUNK_SIZE: (usize, usize, usize) = (32, 32, 32);

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Block {
    pub id: u16,
    pub health: f32,
}

impl Block {
    pub fn new(id: u16, health: f32) -> Block {
        Block {
            id: id,
            health: health,
        }
    }
}

#[non_exhaustive]
struct Faces;
struct Face {
    points: &'static [(i32, i32, i32); 4],
    normal: (i32, i32, i32),
    face_id: u8,
}

impl Faces {
    pub const RIGHT: &'static Face = &Face {
        points: &[(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)],
        normal: (1, 0, 0),
        face_id: 0,
    };
    pub const LEFT: &'static Face = &Face {
        points: &[(0, 0, 1), (0, 1, 1), (0, 1, 0), (0, 0, 0)],
        normal: (-1, 0, 0),
        face_id: 1,
    };
    pub const BOTTOM: &'static Face = &Face {
        points: &[(1, 0, 0), (1, 0, 1), (0, 0, 1), (0, 0, 0)],
        normal: (0, -1, 0),
        face_id: 2,
    };
    pub const TOP: &'static Face = &Face {
        points: &[(1, 1, 1), (1, 1, 0), (0, 1, 0), (0, 1, 1)],
        normal: (0, 1, 0),
        face_id: 3,
    };
    pub const FRONT: &'static Face = &Face {
        points: &[(1, 0, 1), (1, 1, 1), (0, 1, 1), (0, 0, 1)],
        normal: (0, 0, 1),
        face_id: 4,
    };
    pub const BACK: &'static Face = &Face {
        points: &[(0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 0, 0)],
        normal: (0, 0, -1),
        face_id: 5,
    };
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: u32,
    tex_coords: u32,
}

implement_vertex!(Vertex, position, tex_coords);

pub struct Chunk {
    block_data: Option<Box<ndarray::Array3<Block>>>,
    needs_update: bool,
}

impl Chunk {
    pub fn empty(position: (i32, i32, i32)) -> Chunk {
        Chunk {
            block_data: None,
            needs_update: false,
        }
    }

    pub fn new(position: (i32, i32, i32)) -> Chunk {
        Chunk {
            block_data: None,
            needs_update: true,
        }
    }

    fn add_face(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u16>,
        (i, j, k): (usize, usize, usize),
        face: &Face,
    ) {
        const FACE_INDICES: &[i32; 6] = &[2, 1, 0, 0, 3, 2];
        const TEX_COORDS: &[[f32; 2]; 4] = &[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let mut mesh_face_index_loc: [usize; 4] = [0; 4];
        // let tex_coords = Block::get_texture_coords(1, face.face_id);
        let face_tex_coords = { {} };

        for c in 0..4 {
            let (fx, fy, fz) = face.points.get(c).unwrap();
            let point_in_chunk_space = (
                i as i32 + fx,
                j as i32 + fy,
                k as i32 + fz,
            );
            mesh_face_index_loc[c] = vertices.len() as usize;

            vertices.push(Vertex {
                position: 
                    (point_in_chunk_space.0 as u32) |
                    (point_in_chunk_space.1 as u32) << 6 |
                    (point_in_chunk_space.2 as u32) << 12 | 
                    (face.normal.0 as u32) << 18 |
                    (face.normal.1 as u32) << 19 |
                    (face.normal.2 as u32) << 20
                ,
                tex_coords: match face.face_id {
                    0 | 1 | 2 | 3 | 4 | 5 | _ => (TEX_COORDS[c][0] * 1000.0) as u32 | ((TEX_COORDS[c][1] * 1000.0) as u32) << 16,
                },
            });
        }

        for ind in FACE_INDICES.iter() {
            indices.push(mesh_face_index_loc[*ind as usize] as u16);
        }
    }

    pub fn gen_mesh(
        &self,
        neighbors: (
            Option<&Chunk>,
            Option<&Chunk>,
            Option<&Chunk>,
            Option<&Chunk>,
            Option<&Chunk>,
            Option<&Chunk>,
        ),
    ) -> Option<(Vec::<Vertex>, Vec::<u16>)> {
        if !self.needs_update || neighbors.0.is_none() || neighbors.1.is_none() || neighbors.2.is_none() || neighbors.3.is_none() || neighbors.4.is_none() || neighbors.5.is_none() {
            return None;
        }

        let mut vertices = vec![];
        let mut indices = vec![];

        for i in 0..CHUNK_SIZE.0 {
            for j in 0..CHUNK_SIZE.1 {
                for k in 0..CHUNK_SIZE.2 {
                    // Check if block or air
                    if self.block_data.as_ref().unwrap().get((i, j, k)).unwrap().id != 0 {
                        // Check adjacent blocks

                        // Add right face to mesh
                        if i == CHUNK_SIZE.0 - 1
                            || self
                                .block_data
                                .as_ref()
                                .unwrap()
                                .get((i + 1, j, k))
                                .unwrap()
                                .id
                                == 0
                        {
                            // Check neighbor chunk if block is on edge
                            if i == CHUNK_SIZE.0 - 1 {
                                if neighbors.0.is_some()
                                    && (neighbors.0.unwrap().block_data.is_none()
                                        || neighbors
                                            .0
                                            .unwrap()
                                            .block_data
                                            .as_ref()
                                            .unwrap()
                                            .get((0, j, k))
                                            .unwrap_or(&Block { id: 0, health: 0.0 })
                                            .id
                                            == 0)
                                {
                                    self.add_face(
                                        &mut vertices,
                                        &mut indices,
                                        (i, j, k),
                                        Faces::RIGHT,
                                    );
                                }
                            } else {
                                self.add_face(&mut vertices, &mut indices, (i, j, k), Faces::RIGHT);
                            }
                        }

                        // Add left face to mesh
                        if i == 0
                            || self
                                .block_data
                                .as_ref()
                                .unwrap()
                                .get((i - 1, j, k))
                                .unwrap()
                                .id
                                == 0
                        {
                            if i == 0 {
                                if neighbors.1.is_some()
                                    && (neighbors.1.unwrap().block_data.is_none()
                                        || neighbors
                                            .1
                                            .unwrap()
                                            .block_data
                                            .as_ref()
                                            .unwrap()
                                            .get((CHUNK_SIZE.0 - 1, j, k))
                                            .unwrap_or(&Block { id: 0, health: 0.0 })
                                            .id
                                            == 0)
                                {
                                    self.add_face(
                                        &mut vertices,
                                        &mut indices,
                                        (i, j, k),
                                        Faces::LEFT,
                                    );
                                }
                            } else {
                                self.add_face(&mut vertices, &mut indices, (i, j, k), Faces::LEFT);
                            }
                        }

                        // Add bottom face to mesh
                        if j == 0
                            || self
                                .block_data
                                .as_ref()
                                .unwrap()
                                .get((i, j - 1, k))
                                .unwrap()
                                .id
                                == 0
                        {
                            if j == 0 {
                                if neighbors.2.is_some()
                                    && (neighbors.2.unwrap().block_data.is_none()
                                        || neighbors
                                            .2
                                            .unwrap()
                                            .block_data
                                            .as_ref()
                                            .unwrap()
                                            .get((i, CHUNK_SIZE.1 - 1, k))
                                            .unwrap_or(&Block { id: 0, health: 0.0 })
                                            .id
                                            == 0)
                                {
                                    self.add_face(
                                        &mut vertices,
                                        &mut indices,
                                        (i, j, k),
                                        Faces::BOTTOM,
                                    );
                                }
                            } else {
                                self.add_face(
                                    &mut vertices,
                                    &mut indices,
                                    (i, j, k),
                                    Faces::BOTTOM,
                                );
                            }
                        }

                        // Add top face to mesh
                        if j == CHUNK_SIZE.1 - 1
                            || self
                                .block_data
                                .as_ref()
                                .unwrap()
                                .get((i, j + 1, k))
                                .unwrap()
                                .id
                                == 0
                        {
                            if j == CHUNK_SIZE.1 - 1 {
                                if neighbors.3.is_some()
                                    && (neighbors.3.unwrap().block_data.is_none()
                                        || neighbors
                                            .3
                                            .unwrap()
                                            .block_data
                                            .as_ref()
                                            .unwrap()
                                            .get((i, 0, k))
                                            .unwrap_or(&Block { id: 0, health: 0.0 })
                                            .id
                                            == 0)
                                {
                                    self.add_face(
                                        &mut vertices,
                                        &mut indices,
                                        (i, j, k),
                                        Faces::TOP,
                                    );
                                }
                            } else {
                                self.add_face(&mut vertices, &mut indices, (i, j, k), Faces::TOP);
                            }
                        }

                        // Add front face to mesh
                        if k == CHUNK_SIZE.2 - 1
                            || self
                                .block_data
                                .as_ref()
                                .unwrap()
                                .get((i, j, k + 1))
                                .unwrap()
                                .id
                                == 0
                        {
                            if k == CHUNK_SIZE.2 - 1 {
                                if neighbors.4.is_some()
                                    && (neighbors.4.unwrap().block_data.is_none()
                                        || neighbors
                                            .4
                                            .unwrap()
                                            .block_data
                                            .as_ref()
                                            .unwrap()
                                            .get((i, j, 0))
                                            .unwrap_or(&Block { id: 0, health: 0.0 })
                                            .id
                                            == 0)
                                {
                                    self.add_face(
                                        &mut vertices,
                                        &mut indices,
                                        (i, j, k),
                                        Faces::FRONT,
                                    );
                                }
                            } else {
                                self.add_face(&mut vertices, &mut indices, (i, j, k), Faces::FRONT);
                            }
                        }

                        // Add back face to mesh
                        if k == 0
                            || self
                                .block_data
                                .as_ref()
                                .unwrap()
                                .get((i, j, k - 1))
                                .unwrap()
                                .id
                                == 0
                        {
                            if k == 0 {
                                if neighbors.5.is_some()
                                    && (neighbors.5.unwrap().block_data.is_none()
                                        || neighbors
                                            .5
                                            .unwrap()
                                            .block_data
                                            .as_ref()
                                            .unwrap()
                                            .get((i, j, CHUNK_SIZE.2 - 1))
                                            .unwrap_or(&Block { id: 0, health: 0.0 })
                                            .id
                                            == 0)
                                {
                                    self.add_face(
                                        &mut vertices,
                                        &mut indices,
                                        (i, j, k),
                                        Faces::BACK,
                                    );
                                }
                            } else {
                                self.add_face(&mut vertices, &mut indices, (i, j, k), Faces::BACK);
                            }
                        }
                    }
                }
            }
        }

        Some((vertices, indices))
    }

    pub fn set_block(&mut self, (i, j, k): (usize, usize, usize), block: Block) {
        match self.block_data {
            None => self.block_data = Some(Box::new(ndarray::Array3::default(CHUNK_SIZE))),
            Some(_) => {
                if self.block_data.as_ref().unwrap()[[i,j,k]] != block {
                    self.needs_update = true;
                }
            },
        }
        
        self.block_data.as_mut().unwrap()[[i, j, k]] = block;
    }

    pub fn get_block(&self, (i, j, k): (usize, usize, usize)) -> Option<&Block> {
        match &self.block_data {
            None => None,
            Some(data) => data.get((i, j, k)),
        }
    }

    // pub fn get_pos(&self) -> (i32, i32, i32) {
    //     self.position
    // }

    pub fn get_data_mut(&mut self) -> &mut Option<Box<ndarray::Array3<Block>>> {
        &mut self.block_data
    }

    pub fn set_updated(&mut self) {
        self.needs_update = false;
    }

    pub fn is_empty(&self) -> bool {
        self.block_data.is_none()
    }

    pub fn needs_update(&self) -> bool {
        self.needs_update
    }
}
