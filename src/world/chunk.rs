use glam::{Mat4, Vec3};

use crate::gpu::{GpuWrapper, Mesh};

use super::block::{Block, BlockModel, SolidBlockUv};

pub struct Chunk {
    blocks: [Block; 16 * 16 * 128],
    mesh: Option<Mesh>,
    mesh_outdated: bool,
    pub uniform_offset: wgpu::DynamicOffset,
    pub uniform_index: Option<usize>,
    transform: Mat4,
}

impl Chunk {
    pub fn new(
        x: i32,
        z: i32,
        uniform_offset: wgpu::DynamicOffset,
        uniform_index: Option<usize>,
    ) -> Chunk {
        Chunk {
            blocks: [Block::Air; 16 * 16 * 128],
            mesh: None,
            mesh_outdated: true,
            uniform_offset,
            uniform_index,
            transform: Mat4::from_translation(Vec3::new(x as f32 * 16., 0., z as f32 * 16.)),
        }
    }

    pub fn block(&self, x: usize, y: usize, z: usize) -> Block {
        debug_assert!(x < 16, "x = {}", x);
        debug_assert!(y < 256, "y = {}", y);
        debug_assert!(z < 16, "z = {}", z);
        self.blocks[(y * 16 + z) * 16 + x]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        debug_assert!(x < 16, "x = {}", x);
        debug_assert!(y < 256, "y = {}", y);
        debug_assert!(z < 16, "z = {}", z);
        self.mesh_outdated = true;
        self.blocks[(y * 16 + z) * 16 + x] = block;
    }

    pub fn rebuild_mesh(&mut self, gpu: &GpuWrapper) {
        self.mesh_outdated = false;
        let mut vertices = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for x in 0..16 {
            for y in 0..128 {
                for z in 0..16 {
                    let block = self.block(x, y, z);
                    if let Block::Air = block {
                        continue;
                    }

                    match block.model() {
                        BlockModel::Solid(solid) => self.build_full_block_model(
                            x,
                            y,
                            z,
                            &mut vertices,
                            &mut indices,
                            &solid,
                        ),
                    }
                }
            }
        }
        let _ = self.mesh.insert(gpu.create_mesh(&vertices, &indices));
    }

    pub fn build_full_block_model(
        &self,
        x: usize,
        y: usize,
        z: usize,
        vertices: &mut Vec<ChunkVertex>,
        indices: &mut Vec<u32>,
        solid: &SolidBlockUv,
    ) {
        let xf = x as f32;
        let yf = y as f32;
        let zf = z as f32;

        if x == 0 || self.block(x - 1, y, z) == Block::Air {
            let (u, v, ue, ve) = solid.neg_x;
            let n = vertices.len() as u32;
            vertices.extend_from_slice(&[
                ChunkVertex {
                    pos: [xf, yf, zf],
                    uv: [ue, ve],
                },
                ChunkVertex {
                    pos: [xf, yf + 1., zf],
                    uv: [ue, v],
                },
                ChunkVertex {
                    pos: [xf, yf + 1., zf + 1.],
                    uv: [u, v],
                },
                ChunkVertex {
                    pos: [xf, yf, zf + 1.],
                    uv: [u, ve],
                },
            ]);

            indices.extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 3, n]);
        }

        if x == 15 || self.block(x + 1, y, z) == Block::Air {
            let (u, v, ue, ve) = solid.pos_x;
            let n = vertices.len() as u32;
            vertices.extend_from_slice(&[
                ChunkVertex {
                    pos: [xf + 1., yf, zf],
                    uv: [u, ve],
                },
                ChunkVertex {
                    pos: [xf + 1., yf, zf + 1.],
                    uv: [ue, ve],
                },
                ChunkVertex {
                    pos: [xf + 1., yf + 1., zf + 1.],
                    uv: [ue, v],
                },
                ChunkVertex {
                    pos: [xf + 1., yf + 1., zf],
                    uv: [u, v],
                },
            ]);

            indices.extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 3, n]);
        }

        if y == 0 || self.block(x, y - 1, z) == Block::Air {
            let (u, v, ue, ve) = solid.neg_y;
            let n = vertices.len() as u32;
            vertices.extend_from_slice(&[
                ChunkVertex {
                    pos: [xf, yf, zf],
                    uv: [u, v],
                },
                ChunkVertex {
                    pos: [xf, yf, zf + 1.],
                    uv: [u, ve],
                },
                ChunkVertex {
                    pos: [xf + 1., yf, zf + 1.],
                    uv: [ue, ve],
                },
                ChunkVertex {
                    pos: [xf + 1., yf, zf],
                    uv: [ue, v],
                },
            ]);

            indices.extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 3, n]);
        }

        if y == 127 || self.block(x, y + 1, z) == Block::Air {
            let (u, v, ue, ve) = solid.pos_y;
            let n = vertices.len() as u32;
            vertices.extend_from_slice(&[
                ChunkVertex {
                    pos: [xf, yf + 1., zf],
                    uv: [u, v],
                },
                ChunkVertex {
                    pos: [xf + 1., yf + 1., zf],
                    uv: [ue, v],
                },
                ChunkVertex {
                    pos: [xf + 1., yf + 1., zf + 1.],
                    uv: [ue, ve],
                },
                ChunkVertex {
                    pos: [xf, yf + 1., zf + 1.],
                    uv: [u, ve],
                },
            ]);

            indices.extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 3, n]);
        }

        if z == 0 || self.block(x, y, z - 1) == Block::Air {
            let (u, v, ue, ve) = solid.neg_z;
            let n = vertices.len() as u32;
            vertices.extend_from_slice(&[
                ChunkVertex {
                    pos: [xf, yf, zf],
                    uv: [u, ve],
                },
                ChunkVertex {
                    pos: [xf + 1., yf, zf],
                    uv: [ue, ve],
                },
                ChunkVertex {
                    pos: [xf + 1., yf + 1., zf],
                    uv: [ue, v],
                },
                ChunkVertex {
                    pos: [xf, yf + 1., zf],
                    uv: [u, v],
                },
            ]);

            indices.extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 3, n]);
        }

        if z == 15 || self.block(x, y, z + 1) == Block::Air {
            let (u, v, ue, ve) = solid.pos_z;
            let n = vertices.len() as u32;
            vertices.extend_from_slice(&[
                ChunkVertex {
                    pos: [xf, yf, zf + 1.],
                    uv: [ue, ve],
                },
                ChunkVertex {
                    pos: [xf, yf + 1., zf + 1.],
                    uv: [ue, v],
                },
                ChunkVertex {
                    pos: [xf + 1., yf + 1., zf + 1.],
                    uv: [u, v],
                },
                ChunkVertex {
                    pos: [xf + 1., yf, zf + 1.],
                    uv: [u, ve],
                },
            ]);

            indices.extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 3, n]);
        }
    }

    pub fn mesh(&self) -> Option<&Mesh> {
        self.mesh.as_ref()
    }

    pub fn needs_mesh_rebuild(&self) -> bool {
        self.mesh_outdated
    }

    pub fn transform(&self) -> Mat4 {
        self.transform
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}

const CHUNK_VERTEX_ATTRIBS: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

impl crate::gpu::VertexAttribues for ChunkVertex {
    fn attributes() -> &'static [wgpu::VertexAttribute] {
        &CHUNK_VERTEX_ATTRIBS
    }
}

pub fn to_chunk_pos(x: i32, z: i32) -> (i32, i32) {
    (
        (x as f32 / 16.).floor() as i32,
        (z as f32 / 16.).floor() as i32,
    )
}

pub fn world_to_chunk_relative(x: i32, z: i32) -> (i32, i32) {
    ((16 + (x % 16)) % 16, (16 + (z % 16)) % 16)
}
