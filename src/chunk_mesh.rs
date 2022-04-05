use crate::chunk::*;

pub struct ChunkMesh {
    mesh: glium::VertexBuffer<Vertex>,
    indices: glium::IndexBuffer<u16>,
}

impl ChunkMesh {
    pub fn new(mesh: glium::VertexBuffer<Vertex>, indices: glium::IndexBuffer<u16>) -> Self {
        ChunkMesh {
            mesh: mesh,
            indices: indices,
        }
    }

    pub fn get_mesh(&self) -> &glium::VertexBuffer<Vertex> {
        &self.mesh
    }

    pub fn get_indices(&self) -> &glium::IndexBuffer<u16> {
        &self.indices
    }
}
