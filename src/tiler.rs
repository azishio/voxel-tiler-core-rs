use std::fmt::Debug;
use std::io::{BufReader, Seek};

use coordinate_transformer::{jpr2ll, JprOrigin, ll2pixel, pixel2ll, pixel_resolution, ZoomLv};
use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use las::{Color, Read, Reader};
use vec_x::VecX;

use crate::{Coord, PixelPointCloud, Point, RGB, VoxelCollection, VoxelMesh};

pub struct VoxelTile {
    pub voxel_mesh: VoxelMesh<f32>,
    pub tile_idx: VecX<u32, 2>,
}

pub struct VoxelTiler {}

impl VoxelTiler {
    #[cfg(feature = "las")]
    pub fn from_jpr_las<T: std::io::Read + Seek + Send + Debug>(las: T, jpr_origin: JprOrigin, zoom_lv_list: Vec<ZoomLv>, rotate: bool) -> Vec<(ZoomLv, Vec<VoxelTile>)> {
        let mut reader = Reader::new(BufReader::new(las)).unwrap();


        let zoom_lv_set = IndexSet::<ZoomLv, FxBuildHasher>::from_iter(zoom_lv_list);

        zoom_lv_set.into_iter().map(|zoom_lv| {
            let jpr_points = reader.points().map(|wrapped_points| {
                let point = wrapped_points.unwrap();

                // 時々Lasファイルでのxが平面直角座標系のyになっていることがあるので、rotate引数で対応
                let (long, lat) = if !rotate { jpr2ll((point.y, point.x), jpr_origin) } else { jpr2ll((point.x, point.y), jpr_origin) };

                let (x, y) = ll2pixel((long, lat), zoom_lv);

                let pixel_resolution = pixel_resolution(lat, zoom_lv);

                let z = (point.z / pixel_resolution) as u32;

                let color = point.color.unwrap_or(Color::new(0, 0, 0));

                let r = (color.red / u8::MAX as u16) as u8;
                let g = (color.green / u8::MAX as u16) as u8;
                let b = (color.blue / u8::MAX as u16) as u8;

                (Coord::new([x, y, z]), RGB::new([r, g, b]))
            }).collect::<Vec<Point<u32>>>();

            let point_cloud = PixelPointCloud::new(jpr_points, zoom_lv);

            let voxel_tile_list = Self::from_pixel_point_cloud(point_cloud);

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
