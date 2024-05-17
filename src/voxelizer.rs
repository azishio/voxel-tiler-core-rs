use std::fmt::Debug;
use std::io::{BufRead, Seek};

use coordinate_transformer::{jpr2ll, JprOrigin, ll2pixel, pixel2ll, pixel_resolution, ZoomLv};
#[cfg(feature = "las")]
use las::{Color, Read, Reader};
use vec_x::VecX;

use crate::{Coord, PixelPointCloud, Point, RGB, TileIdx, VoxelCollection, VoxelMesh};

pub struct VoxelModel {
    pub voxel_mesh: VoxelMesh<f32>,
    // 左上端のピクセルが属するタイルのインデックス
    pub origin_tile_idx: VecX<u32, 2>,
}

pub struct Voxelizer {}

impl Voxelizer {
    #[cfg(feature = "las")]
    pub fn from_jpr_las<T: BufRead + Seek + Send + Debug>(las: T, jpr_origin: JprOrigin, zoom_lv: ZoomLv, threshold: usize, rotate: bool) -> VoxelModel {
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

        Self::from_pixel_point_cloud(point_cloud, threshold)
    }

    pub fn from_pixel_point_cloud(point_cloud: PixelPointCloud, threshold: usize) -> VoxelModel {
        let (min_x, min_y) = point_cloud.points.iter().fold((u32::MAX, u32::MAX), |(min_x, min_y), (pixel_coord, _)| (min_x.min(pixel_coord[0]), min_y.min(pixel_coord[1])));
        let min_tile_idx = VecX::new([min_x / 256, min_y / 256]);

        let voxel_collection = VoxelCollection::from_pixel_point_cloud(point_cloud, threshold);

        Self::from_voxel_collection(min_tile_idx, voxel_collection)
    }

    pub fn from_voxel_collection(min_tile_idx: TileIdx, voxel_collection: VoxelCollection) -> VoxelModel {
        let f = |v: Coord<u32>, zoom_lv: ZoomLv| -> Coord<f32>{
            let voxel_size = {
                let (_, lat) = pixel2ll((v[0], v[1]), zoom_lv);
                pixel_resolution(lat, zoom_lv)
            };


            //   (ピクセル座標 - タイル座標 * 1タイルのピクセル数) * ボクセルサイズ
            // = (タイル右上を原点としたローカルのピクセル座標) * ボクセルサイズ
            // = タイル右上を原点とした点の位置(m)
            let x = ((v[0] - (min_tile_idx[0] * 256)) as f64 * voxel_size) as f32;
            let y = ((v[1] - (min_tile_idx[1] * 256)) as f64 * voxel_size) as f32;
            let z = (v[2] as f64 * voxel_size) as f32;

            VecX::new([x, y, z])
        };

        let voxel_mesh = VoxelMesh::<u32>::from_voxel_collection(voxel_collection).coordinate_transform(f);

        VoxelModel {
            voxel_mesh,
            origin_tile_idx: min_tile_idx,
        }
    }
}
