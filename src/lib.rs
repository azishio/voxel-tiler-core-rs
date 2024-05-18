#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jamalloc")]
use tikv_jemallocator::Jemalloc;

pub use mesh::*;
pub use params::*;
pub use ply::*;
pub use voxel::*;
pub use voxelizer::*;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jamalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod voxel;
mod ply;
mod mesh;
mod voxelizer;
mod params;

