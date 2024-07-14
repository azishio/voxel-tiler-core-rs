#![cfg_attr(docsrs, feature(doc_cfg))]

pub use coordinate_transformer;
pub use gltf;
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
#[cfg(feature = "image")]
pub use image;
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
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
#[cfg(feature = "image")]
pub mod image_sampler;
pub mod glb;
#[cfg_attr(docsrs, doc(cfg(feature = "ply")))]
#[cfg(feature = "ply")]
pub mod ply;
pub mod mesh;

/// lasファイルから点群を読むためのモジュールです。
/// 使用するには`las`featureを有効にしてください。
#[cfg_attr(docsrs, doc(cfg(feature = "las")))]
#[cfg(feature = "las")]
mod las;
