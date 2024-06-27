#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jamalloc")]
use tikv_jemallocator::Jemalloc;

pub use params::*;
pub use voxelizer::*;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jamalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub mod voxelizer;
pub mod params;
pub mod build_voxelizer;
pub mod collection;
pub mod element;
pub mod image_sampler;
pub mod mesher;
pub mod glb_gen;
pub mod ply;

