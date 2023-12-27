#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Block {
    Air,
    Stone,
    Grass,
    Dirt,
    Cobblestone,
    Planks,
}

impl Block {
    pub fn read(ty: u8, _data: u8) -> Option<Block> {
        let block = match ty {
            0 => Block::Air,
            1 => Block::Stone,
            2 => Block::Grass,
            3 => Block::Dirt,
            4 => Block::Cobblestone,
            5 => Block::Planks,
            _ => return None,
        };
        Some(block)
    }

    pub fn model(self) -> BlockModel {
        match self {
            Block::Air => unreachable!(),
            Block::Stone => SolidBlockUv::all(1./16., 0.),
            Block::Grass => SolidBlockUv::top_side_bottom(0., 0., 3./16., 0., 2./16., 0./16.),
            Block::Dirt => SolidBlockUv::all(2./16., 0.),
            Block::Cobblestone => SolidBlockUv::all(0., 1./16.),
            Block::Planks => SolidBlockUv::all(4./16., 0.),
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

    pub fn top_side_bottom(top_u: f32, top_v: f32, side_u: f32, side_v: f32, bottom_u: f32, bottom_v: f32) -> BlockModel {
        let side_uv = (side_u, side_v, side_u + 1./16., side_v + 1./16.);
        BlockModel::Solid(
            SolidBlockUv {
                pos_x: side_uv,
                neg_x: side_uv,
                pos_y: (top_u, top_v, top_u + 1./16., top_v + 1./16.),
                neg_y: (bottom_u, bottom_v, bottom_u + 1./16., bottom_v + 1./16.),
                pos_z: side_uv,
                neg_z: side_uv,
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
