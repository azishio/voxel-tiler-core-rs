#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Offset {
    Tile,
    MinTile,
    Pixel,
    Voxel,
}

pub trait VoxelizerParams {
    const TILING: bool;
    const THRESHOLD: usize;
    const OFFSET: Offset;
}

pub mod default_params {
    use super::*;

    pub struct Tile;

    impl VoxelizerParams for Tile {
        const TILING: bool = true;
        const THRESHOLD: usize = 1;
        const OFFSET: Offset = Offset::Tile;
    }

    pub struct Fit;

    impl VoxelizerParams for Fit {
        const TILING: bool = false;
        const THRESHOLD: usize = 1;
        const OFFSET: Offset = Offset::Voxel;
    }
}
