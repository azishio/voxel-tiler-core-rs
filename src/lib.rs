#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jamalloc")]
use tikv_jemallocator::Jemalloc;

pub use build_voxelizer::*;
pub use collection::*;
pub use element::*;
pub use glb_gen::*;
pub use image_sampler::*;
#[cfg(feature = "las")]
pub use las::*;
pub use mesher::*;
#[cfg(feature = "ply")]
pub use ply::*;
pub use voxel_mesh::*;
pub use voxelizer::*;

#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jamalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub mod voxelizer;
pub mod build_voxelizer;
pub mod collection;
pub mod element;
pub mod image_sampler;
pub mod mesher;
pub mod glb_gen;
#[cfg(feature = "ply")]
pub mod ply;
pub mod voxel_mesh;
#[cfg(feature = "las")]
pub mod las;
