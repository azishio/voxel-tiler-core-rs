use std::collections::HashSet;
use std::hash::Hash;

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use num::Num;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use vec_x::VecX;

use crate::{Point, VoxelCollection};

pub type VertexIndices = Vec<usize>;

type Coord<T> = VecX<T, 3>;

/// a structure that holds information about the mesh representing the voxel
///
/// ボクセルを表現するメッシュの情報を保持する構造体
#[derive(Clone, Debug)]
pub struct VoxelMesh<T>
    where T: Num + Sized + Send
{
    /// Unique (coordinates,color) list
    ///
    /// 一意な(座標,色)のリスト
    pub vertices: Vec<Point<T>>,

    /// List of indices of the vertices that make up the face
    ///
    /// 面を構成する頂点のインデックスのリスト
    pub face: Vec<VertexIndices>,
}

impl<T> VoxelMesh<T>
    where
        T: Num + Sized + Send + Copy + Eq + Hash
{
    /// Generate a mesh
    ///
    /// メッシュを生成する
    pub fn new(vertices: Vec<Point<T>>, face: Vec<VertexIndices>) -> Self {
        Self {
            vertices,
            face,
        }
    }

    /// Generate an empty mesh
    ///
    /// 空のメッシュを生成する
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            face: Vec::new(),
        }
    }

    /// Converts voxel data into a list of rectangular polygons that retain their detail.
    ///
    /// ボクセルデータを、そのディティールを保持した四角形ポリゴンのリストに変換します。
    pub fn from_voxel_collection(voxel_collection: VoxelCollection) -> VoxelMesh<u32> {
        let voxels = voxel_collection.voxels;

        let voxel_set = HashSet::<Coord<u32>, FxBuildHasher>::from_iter(voxels.iter().map(|(pixel_coord, _)| *pixel_coord));

        let mut vertex_set = IndexSet::<Point<u32>, FxBuildHasher>::with_hasher(FxBuildHasher::default());

        let face_list = voxels.into_iter().flat_map(|(pixel_coord, rgb)| {
            let x = pixel_coord[0];
            let y = pixel_coord[1];
            let z = pixel_coord[2];

            [
                ([(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)], voxel_set.contains(&Coord::new([x + 1, y, z]))),
                ([(0, 0, 0), (0, 1, 0), (0, 1, 1), (0, 0, 1)], voxel_set.contains(&Coord::new([x - 1, y, z]))),
                ([(0, 1, 0), (1, 1, 0), (1, 1, 1), (0, 1, 1)], voxel_set.contains(&Coord::new([x, y + 1, z]))),
                ([(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)], voxel_set.contains(&Coord::new([x, y - 1, z]))),
                ([(0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)], voxel_set.contains(&Coord::new([x, y, z + 1]))),
                ([(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0)], voxel_set.contains(&Coord::new([x, y, z - 1]))),
            ].iter().filter_map(|(vertexes, has_adjacent)| {
                if *has_adjacent {
                    None
                } else {
                    let vertex_list = vertexes.iter().map(|(dx, dy, dz)| Coord::new([x + dx, y + dy, z + dz]));

                    let vertex_indices = vertex_list.map(|vertex| vertex_set.insert_full((vertex, rgb)).0).collect::<Vec<_>>();

                    Some(vertex_indices)
                }
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();

        let vertices = vertex_set.into_iter().collect::<Vec<_>>();

        VoxelMesh {
            vertices,
            face: face_list,
        }
    }

    /// Returns the result of applying the specified function to all vertices.
    /// For example, it can be used to transform the coordinate system of a vertex.
    ///
    /// 全ての頂点について、指定された関数を適用した結果を返します。
    /// 例えば、頂点の座標系を変換する場合に使用できます。
    #[cfg(not(feature = "rayon"))]
    pub fn batch_to_vertices<U, F>(self, f: F) -> VoxelMesh<U>
        where
            U: Num + Sized + Send,
            F: Fn(Point<T>) -> Point<U> + Sync + Send,
    {
        let Self { vertices, face } = self;

        let vertices = vertices.into_iter().map(f).collect::<Vec<_>>();

        VoxelMesh {
            vertices,
            face,
        }
    }

    /// Returns the result of applying the specified function to all vertices.
    /// For example, it can be used to transform the coordinate system of a vertex.
    ///
    /// 全ての頂点について、指定された関数を適用した結果を返します。
    /// 例えば、頂点の座標系を変換する場合に使用できます。
    #[cfg(feature = "rayon")]
    pub fn batch_to_vertices<U, F>(self, f: F) -> VoxelMesh<U>
        where
            U: Num + Sized + Send,
            F: Fn(Point<T>) -> Point<U> + Sync + Send,
    {
        let Self { vertices, face } = self;

        let vertices = vertices.into_par_iter().map(f).collect::<Vec<_>>();

        VoxelMesh {
            vertices,
            face,
        }
    }
}
