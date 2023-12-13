use crate::gpu::{Mesh, GpuWrapper};

pub struct Chunk {
    blocks: [bool; 16*16*128],
    mesh: Option<Mesh>,
    x: i32,
    z: i32,
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Chunk {
        Chunk {
            blocks: [false; 16*16*128],
            mesh: None,
            x,
            z,
        }
    }

    pub fn block(&self, x: usize, y: usize, z: usize) -> bool {
        self.blocks[(y * 16 + z) * 16 + x]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: bool) {
        self.blocks[(y * 16 + z) * 16 + x] = block;
    }

    pub fn rebuild_mesh(&mut self, gpu: &GpuWrapper) {
        self.set_block(0, 0, 0, true);
        self.set_block(0, 0, 3, true);

        let mut vertices = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        /*for x in 0..16 {
            for y in 0..128 {
                for z in 0..16 {
                    let index = (y * 16 + z) * 16 + x;
                    if !self.blocks[index] {
                        continue
                    }

                    let xf = x as f32;
                    let yf = y as f32;
                    let zf = z as f32;

                    //if x == 0 || !self.block(x-1, y, z) {
                        vertices.extend_from_slice(&[
                            ChunkVertex {
                                pos: [xf, yf, zf],
                                uv: [0., 1.],
                            },
                            ChunkVertex {
                                pos: [xf, yf + 1., zf],
                                uv: [1., 1.],
                            },
                            ChunkVertex {
                                pos: [xf, yf + 1., zf + 1.],
                                uv: [1., 0.],
                            },
                            ChunkVertex {
                                pos: [xf, yf, zf + 1.],
                                uv: [0., 0.],
                            },
                        ]);

                        let i = indices.len() as u32;
                        indices.extend_from_slice(&[i, i+1, i+2, i+2, i+3, i]);
                    //}

                    //if z == 0 || !self.block(x, y, z-1) {
                        vertices.extend_from_slice(&[
                            ChunkVertex {
                                pos: [xf, yf, zf],
                                uv: [0., 1.],
                            },
                            ChunkVertex {
                                pos: [xf, yf + 1., zf],
                                uv: [1., 1.],
                            },
                            ChunkVertex {
                                pos: [xf + 1., yf + 1., zf],
                                uv: [1., 0.],
                            },
                            ChunkVertex {
                                pos: [xf + 1., yf, zf],
                                uv: [0., 0.],
                            },
                        ]);

                        let i = indices.len() as u32;
                        indices.extend_from_slice(&[i, i+1, i+2, i+2, i+3, i]);
                    //}

                    //if z == 0 || !self.block(x, y, z-1) {
                        vertices.extend_from_slice(&[
                            ChunkVertex {
                                pos: [xf, yf, zf+1.],
                                uv: [0., 1.],
                            },
                            ChunkVertex {
                                pos: [xf, yf + 1., zf+1.],
                                uv: [1., 1.],
                            },
                            ChunkVertex {
                                pos: [xf + 1., yf + 1., zf+1.],
                                uv: [1., 0.],
                            },
                            ChunkVertex {
                                pos: [xf + 1., yf, zf+1.],
                                uv: [0., 0.],
                            },
                        ]);

                        let i = indices.len() as u32;
                        indices.extend_from_slice(&[i, i+2, i+1, i+2, i, i+3]);
                    //}
                }
            }
        }*/
        vertices.extend_from_slice(&[
            ChunkVertex {
                pos: [0., 0., 0.],
                uv: [0., 1.],
            },
            ChunkVertex {
                pos: [1., 0., 0.],
                uv: [1., 1.],
            },
            ChunkVertex {
                pos: [1., 1., 0.],
                uv: [1., 0.],
            },
            ChunkVertex {
                pos: [0., 1., 0.],
                uv: [0., 0.],
            },
        ]);
        indices.extend_from_slice(&[0, 1, 2, 2, 3, 0]);

        let _ = self.mesh.insert(gpu.create_mesh(&vertices, &indices));
    }

    pub fn mesh(&self) -> Option<&Mesh> {
        self.mesh.as_ref()
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}

const CHUNK_VERTEX_ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

impl crate::gpu::VertexAttribues for ChunkVertex {
    fn attributes() -> &'static [wgpu::VertexAttribute] {
        &CHUNK_VERTEX_ATTRIBS
    }
}
