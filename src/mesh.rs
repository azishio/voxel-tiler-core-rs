use std::collections::HashSet;
use std::hash::Hash;

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use num::Num;
use vec_x::VecX;

use crate::{Point, VoxelCollection};

pub type VertexIndices = Vec<usize>;

type Coord<T> = VecX<T, 3>;

/// ボクセルを表現するメッシュの情報を保持する構造体
///
/// a structure that holds information about the mesh representing the voxel
#[derive(Clone, Debug)]
pub struct VoxelMesh<T: Num> {
    /// 一意な(座標,色)のリスト
    ///
    /// Unique (coordinates,color) list
    pub vertices: Vec<Point<T>>,

    /// 面を構成する頂点のインデックスのリスト
    ///
    /// List of indices of the vertices that make up the face
    pub face: Vec<VertexIndices>,
}

impl<T> VoxelMesh<T>
    where
        T: Num + Copy + Eq + Hash
{
    /// メッシュを生成する
    ///
    /// Generate a mesh
    pub fn new(vertices: Vec<Point<T>>, face: Vec<VertexIndices>) -> Self {
        Self {
            vertices,
            face,
        }
    }

    /// 空のメッシュを生成する
    ///
    /// Generate an empty mesh
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            face: Vec::new(),
        }
    }

    /// ボクセルデータを、そのディティールを保持した四角形ポリゴンのリストに変換します。
    ///
    /// Converts voxel data into a list of rectangular polygons that retain their detail.
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

    /// 全ての頂点について、指定された関数を適用した結果を返します。
    /// 例えば、頂点の座標系を変換する場合に使用できます。
    ///
    /// Returns the result of applying the specified function to all vertices.
    /// For example, it can be used to transform the coordinate system of a vertex.
    pub fn batch_to_vertices<U, F>(self, f: F) -> VoxelMesh<U>
        where
            U: Num,
            F: Fn(Point<T>) -> Point<U>,
    {
        let Self { vertices, face } = self;

        let vertices = vertices.into_iter().map(|point| {
            f(point)
        }).collect::<Vec<_>>();

        VoxelMesh {
            vertices,
            face,
        }
    }
}
