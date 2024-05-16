use std::collections::HashSet;
use std::hash::Hash;

use coordinate_transformer::ZoomLv;
use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use num::Num;
use vec_x::VecX;

use crate::voxel::{RGB, VoxelCollection};

type MaterialIndex = usize;
type VertexIndices = Vec<usize>;

type Coord<T: Num> = VecX<T, 3>;

pub struct VoxelMesh<T: Num> {
    pub vertices: Vec<Coord<T>>,
    pub materials: Vec<RGB>,
    pub face: Vec<(VertexIndices, MaterialIndex)>,
    pub voxel_size: f32,
    pub zoom_lv: ZoomLv,
}

impl<T: Num + Copy + Eq + Hash> VoxelMesh<T> {
    pub fn new(vertices: Vec<Coord<T>>, materials: Vec<RGB>, face: Vec<(VertexIndices, MaterialIndex)>, voxel_size: f32, zoom_lv: ZoomLv) -> Self {
        Self {
            vertices,
            materials,
            face,
            voxel_size,
            zoom_lv,
        }
    }

    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            materials: Vec::new(),
            face: Vec::new(),
            voxel_size: 0.,
            zoom_lv: ZoomLv::Lv0,
        }
    }

    pub fn from_voxel_collection(voxel_collection: VoxelCollection<T>) -> VoxelMesh<T> {
        let VoxelCollection { voxels, voxel_size, zoom_lv } = voxel_collection;

        let voxel_set = HashSet::<Coord<T>, FxBuildHasher>::from_iter(voxels.iter().map(|(pixel_coord, _)| *pixel_coord));

        let mut vertex_set = IndexSet::<Coord<T>, FxBuildHasher>::with_hasher(FxBuildHasher::default());
        let mut material_set = IndexSet::<RGB, FxBuildHasher>::with_hasher(FxBuildHasher::default());

        // ここでメッシュを構成しながら、(頂点,マテリアル)のvecを作成する
        let face_list = voxels.into_iter().flat_map(|(pixel_coord, rgb)| {
            let x = pixel_coord[0];
            let y = pixel_coord[1];
            let z = pixel_coord[2];

            [
                ([(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)], voxel_set.contains(&Coord::new([x + T::one(), y, z]))),
                ([(0, 0, 0), (0, 1, 0), (0, 1, 1), (0, 0, 1)], voxel_set.contains(&Coord::new([x - T::one(), y, z]))),
                ([(0, 1, 0), (1, 1, 0), (1, 1, 1), (0, 1, 1)], voxel_set.contains(&Coord::new([x, y + T::one(), z]))),
                ([(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)], voxel_set.contains(&Coord::new([x, y - T::one(), z]))),
                ([(0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)], voxel_set.contains(&Coord::new([x, y, z + T::one()]))),
                ([(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0)], voxel_set.contains(&Coord::new([x, y, z - T::one()]))),
            ].iter().filter_map(|(vertexes, has_adjacent)| {
                if *has_adjacent {
                    None
                } else {
                    let vertex_list = vertexes.iter().map(|(dx, dy, dz)| Coord::new([x + dx, y + dy, z + dz]));

                    let material_index = material_set.insert_full(rgb).0;
                    let vertex_indices = vertex_list.map(|vertex| vertex_set.insert_full(vertex).0).collect::<Vec<_>>();

                    Some((vertex_indices, material_index))
                }
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();

        let vertices = vertex_set.into_iter().collect::<Vec<_>>();
        let materials = material_set.into_iter().collect::<Vec<_>>();

        VoxelMesh {
            vertices,
            materials,
            face: face_list,
            voxel_size,
            zoom_lv,
        }
    }

    pub fn coordinate_transform<U: Num>(self, f: fn(Coord<T>) -> Coord<U>) -> VoxelMesh<U> {
        let Self { vertices, materials, face, voxel_size, zoom_lv } = self;

        let vertices = vertices.into_iter().map(f).collect::<Vec<_>>();

        VoxelMesh {
            vertices,
            materials,
            face,
            voxel_size,
            zoom_lv,
        }
    }
}
