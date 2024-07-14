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

/// 点群をボクセル化するためのモジュールです。
pub mod voxelizer;
/// 基本的な使用方法にあったデフォルトの設定を適用したボクセライザーを構築するためのモジュールです。
pub mod build_voxelizer;
/// 変換前の点群やボクセルデータを格納する構造体です。
/// 実装によって高速/低速になる処理が違うため、目的によって使い分けてください。
pub mod collection;
/// ボクセルデータを表現するための座標値やRGB色などを表す構造体を定義しています。
pub mod element;
/// 国土地理院が公開する標高タイルを用いてボクセルデータを生成するためのモジュールです。
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
#[cfg(feature = "image")]
pub mod giaj_terrain;
/// glbファイルにメッシュを書き込むためのモジュールです。
pub mod glb;
#[cfg_attr(docsrs, doc(cfg(feature = "ply")))]
#[cfg(feature = "ply")]
pub mod ply;
/// ボクセル化された点群にメッシュを貼るためのモジュール。
pub mod mesh;

/// lasファイルから点群を読むためのモジュールです。
/// 使用するには`las`featureを有効にしてください。
#[cfg_attr(docsrs, doc(cfg(feature = "las")))]
#[cfg(feature = "las")]
mod las;
