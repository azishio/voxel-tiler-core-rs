use vec_x::VecX;

use crate::mesh::VoxelMesh;
use crate::voxel::{PixelPointCloud, VoxelCollection};

pub struct VoxelTile {
    pub voxel_mesh: VoxelMesh<f32>,
    pub tile_idx: VecX<u32, 2>,
}

pub struct VoxelTiler {}

impl VoxelTiler {
    pub fn from_point_cloud(point_cloud: PixelPointCloud) -> Vec<VoxelTile> {
        let tiled_points = point_cloud.split_by_tile();


        tiled_points.into_iter().map(|(tile_idx, pixel_point_cloud)| {
            let voxel_collection = VoxelCollection::from_pixel_point_cloud(pixel_point_cloud);

            let voxel_mesh = VoxelMesh::<u32>::from_voxel_collection(voxel_collection).coordinate_transform(|v, voxel_size, zoom_lv| {
                //   (ピクセル座標 - 2^ズームレベル) * ボクセルサイズ
                // = (ピクセル座標 - タイル原点) * ボクセルサイズ
                // = (タイル右上を原点としたローカルのピクセル座標) * ボクセルサイズ
                // = タイル右上を原点とした点の位置(m)
                let x = (v[0] - (2 ^ zoom_lv as u32)) as f32 * voxel_size;
                let y = (v[1] - (2 ^ zoom_lv as u32)) as f32 * voxel_size;
                let z = (v[2] - (2 ^ zoom_lv as u32)) as f32 * voxel_size;

                VecX::new([x, y, z])
            });

            VoxelTile {
                voxel_mesh,
                tile_idx,
            }
        }).collect::<Vec<_>>()
    }
}
