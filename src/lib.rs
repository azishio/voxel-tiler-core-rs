#[cfg(not(target_env = "msvc"))]
#[cfg(feature = "jamalloc")]
use tikv_jemallocator::Jemalloc;
pub use vec_x;

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
