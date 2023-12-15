#[derive(Clone, Copy, PartialEq)]
pub enum Block {
    Air,
    Stone,
}

impl Block {
    pub fn model(self) -> BlockModel {
        match self {
            Block::Air => unreachable!(),
            Block::Stone => SolidBlockUv::all(1./16., 0.),
        }
    }
}

#[derive(Clone, Copy)]
pub enum BlockModel {
    Solid(SolidBlockUv),
}

/// Coordinates are in order Left, right, top, bottom
#[derive(Default, Clone, Copy)]
pub struct SolidBlockUv {
    pub pos_x: (f32, f32, f32, f32),
    pub neg_x: (f32, f32, f32, f32),
    pub pos_y: (f32, f32, f32, f32),
    pub neg_y: (f32, f32, f32, f32),
    pub pos_z: (f32, f32, f32, f32),
    pub neg_z: (f32, f32, f32, f32),
}

impl SolidBlockUv {
    pub fn all(u: f32, v: f32) -> BlockModel {
        let uv = (u, v, u + 1./16., v + 1./16.);
        BlockModel::Solid(
            SolidBlockUv {
                pos_x: uv,
                neg_x: uv,
                pos_y: uv,
                neg_y: uv,
                pos_z: uv,
                neg_z: uv,
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::world::block::Block;

    #[test]
    fn test_block_size() {
        assert_eq!(std::mem::size_of::<Block>(), 1);
    }
}
