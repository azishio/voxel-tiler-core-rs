use vec_x::VecX;

use crate::{PixelPointCloud, VoxelCollection, VoxelMesh};

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

            (zoom_lv, voxel_tile_list)
        }).collect::<Vec<_>>()
    }

    pub fn from_pixel_point_cloud(point_cloud: PixelPointCloud) -> Vec<VoxelTile> {
        let voxel_collections = VoxelCollection::from_pixel_point_cloud(point_cloud);

        voxel_collections.into_iter().map(|(tile_idx, voxel_collection)| {
            let voxel_mesh = VoxelMesh::<u32>::from_voxel_collection(voxel_collection).coordinate_transform(|v, zoom_lv| {
                let voxel_size = {
                    let (_, lat) = pixel2ll((v[0], v[1]), zoom_lv);
                    pixel_resolution(lat, zoom_lv)
                };

                //   (ピクセル座標 - 2^ズームレベル) * ボクセルサイズ
                // = (ピクセル座標 - タイル原点) * ボクセルサイズ
                // = (タイル右上を原点としたローカルのピクセル座標) * ボクセルサイズ
                // = タイル右上を原点とした点の位置(m)
                let x = ((v[0] - (2 ^ zoom_lv as u32)) as f64 * voxel_size) as f32;
                let y = ((v[1] - (2 ^ zoom_lv as u32)) as f64 * voxel_size) as f32;
                let z = ((v[2] - (2 ^ zoom_lv as u32)) as f64 * voxel_size) as f32;

                VecX::new([x, y, z])
            });

            VoxelTile {
                voxel_mesh,
                tile_idx,
            }
        }).collect::<Vec<_>>()
    }
}
