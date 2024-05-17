use std::fmt::Debug;
use std::io::{BufRead, Seek};

use coordinate_transformer::{jpr2ll, JprOrigin, ll2pixel, pixel_resolution, ZoomLv};
#[cfg(feature = "las")]
use las::{Color, Read, Reader};

use crate::{Coord, PixelPointCloud, Point, RGB, VoxelCollection, Voxelizer, VoxelModel};

pub struct VoxelTiler {}

impl VoxelTiler {
    #[cfg(feature = "las")]
    pub fn from_jpr_las<T: BufRead + Seek + Send + Debug>(las: T, jpr_origin: JprOrigin, zoom_lv: ZoomLv, threshold: usize, rotate: bool) -> Vec<VoxelModel> {
        let mut reader = Reader::new(las).unwrap();


        let points = reader.points().collect::<Vec<_>>();

        let jpr_points = points.into_iter().map(|wrapped_points| {
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

        let voxel_tile_list = Self::from_pixel_point_cloud(point_cloud, threshold);

        voxel_tile_list
    }

    pub fn from_pixel_point_cloud(point_cloud: PixelPointCloud, threshold: usize) -> Vec<VoxelModel> {
        let voxel_collections = VoxelCollection::from_pixel_point_cloud_with_tiling(point_cloud, threshold);

        voxel_collections.into_iter().map(|(tile_idx, voxel_collection)| {
            Voxelizer::from_voxel_collection(tile_idx, voxel_collection)
        }).collect::<Vec<_>>()
    }
}
