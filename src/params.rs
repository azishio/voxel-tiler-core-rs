#[derive(Debug, Copy, Clone)]
pub enum Offset {
    Tile,
    Pixel,
    Voxel,
}

pub trait VoxelizerParam {
    const TILING: bool;
    const THRESHOLD: usize;
    const ROTATE: bool;
    const OFFSET: Offset;
}

pub mod default_params {
    use super::*;

    pub struct Tile;

    impl VoxelizerParam for Tile {
        const TILING: bool = true;
        const THRESHOLD: usize = 1;
        const ROTATE: bool = false;
        const OFFSET: Offset = Offset::Tile;
    }

    pub struct Fit;

    impl VoxelizerParam for Fit {
        const TILING: bool = false;
        const THRESHOLD: usize = 1;
        const ROTATE: bool = false;
        const OFFSET: Offset = Offset::Voxel;
    }
}
