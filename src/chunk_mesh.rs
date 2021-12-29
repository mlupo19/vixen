use crate::chunk::*;

pub struct ChunkMesh {
    mesh: Option<glium::VertexBuffer<Vertex>>,
    indices: Option<glium::IndexBuffer<u16>>,
}

impl ChunkMesh {
    pub fn new(mesh: Option<glium::VertexBuffer<Vertex>>, indices: Option<glium::IndexBuffer<u16>>) -> Self {
        ChunkMesh {
            mesh: mesh,
            indices: indices,
        }
    }

    pub fn get_mesh(&self) -> &Option<glium::VertexBuffer<Vertex>> {
        &self.mesh
    }

    pub fn get_indices(&self) -> &Option<glium::IndexBuffer<u16>> {
        &self.indices
    }
}