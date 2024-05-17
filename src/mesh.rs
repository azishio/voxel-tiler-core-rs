use std::collections::HashSet;
use std::hash::Hash;

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use num::Num;
use vec_x::VecX;

use crate::{Point, VoxelCollection};

pub type MaterialIndex = usize;
pub type VertexIndices = Vec<usize>;

type Coord<T> = VecX<T, 3>;

pub struct VoxelMesh<T: Num> {
    pub vertices: Vec<Point<T>>,
    pub face: Vec<VertexIndices>,
}

impl<T> VoxelMesh<T>
    where
        T: Num + Copy + Eq + Hash
{
    pub fn new(vertices: Vec<Point<T>>, face: Vec<VertexIndices>) -> Self {
        Self {
            vertices,
            face,
        }
    }

    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            face: Vec::new(),
        }
    }

    pub fn from_voxel_collection(voxel_collection: VoxelCollection) -> VoxelMesh<u32> {
        let voxels = voxel_collection.voxels;

        let voxel_set = HashSet::<Coord<u32>, FxBuildHasher>::from_iter(voxels.iter().map(|(pixel_coord, _)| *pixel_coord));

        let mut vertex_set = IndexSet::<Point<u32>, FxBuildHasher>::with_hasher(FxBuildHasher::default());

        // ここでメッシュを構成しながら、(頂点,マテリアル)のvecを作成する
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

    pub fn coordinate_transform<U, F>(self, f: F) -> VoxelMesh<U>
        where
            U: Num,
            F: Fn(Coord<T>) -> Coord<U>,
    {
        let Self { vertices, face } = self;

        let vertices = vertices.into_iter().map(|(coord, material)| {
            let coord = f(coord);
            (coord, material)
        }).collect::<Vec<_>>();

        VoxelMesh {
            vertices,
            face,
        }
    }
}
