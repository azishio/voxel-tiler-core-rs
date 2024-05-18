use std::fmt::Debug;
use std::io::{BufRead, Seek};
use std::marker::PhantomData;

#[cfg(feature = "print-log")]
use chrono::Local;
use coordinate_transformer::{jpr2ll, JprOrigin, ll2pixel, pixel2ll, pixel_resolution, ZoomLv};
#[cfg(feature = "las")]
use las::{Color, Read, Reader};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::{Coord, default_params, Offset, Point, RGB, TileIdx, VoxelCollection, VoxelizerParams, VoxelMesh, VoxelPointCloud};

#[cfg(feature = "print-log")]
fn print_log(log: &str) {
    let time = Local::now().format("%H:%M:%S%.3f");
    println!("{}: {}", time, log);
}

#[cfg(not(feature = "print-log"))]
fn print_log(_: &str) {}


/// The reference implementation for creating voxel data using this crate.
/// You can use this structure for simple applications, but you will need your own implementation if you want to perform more advanced processing.
///
///　本クレートを使用してボクセルデータを作成するリファレンス実装です。
/// 単純な用途であればこの構造体を使用できますが、より高度な処理を行いたい場合は、独自の実装が必要になります。
///
/// Examples are available in `examples/`.
///
/// 使用例は`examples/`にあります。
pub struct Voxelizer<Params: VoxelizerParams = default_params::Fit> {
    _param: PhantomData<Params>,
}

impl<Params: VoxelizerParams> Voxelizer<Params> {
    #[cfg(feature = "las")]
    #[cfg(not(feature = "rayon"))]
    pub fn voxelize_from_jpr_las<T>(las: T, jpr_origin: JprOrigin, zoom_lv: ZoomLv, rotate: bool) -> Vec<(TileIdx, VoxelMesh<f32>)>
        where T: BufRead + Seek + Send + Debug,
    {
        print_log("[log] Voxelizer: Start reading las file");
        let mut reader = Reader::new(las).unwrap();
        let points = reader.points().collect::<Vec<_>>();
        print_log("[log] Voxelizer: Finish reading las file");


        print_log("[log] Voxelizer: Start converting las to point list");
        let jpr_points = points.into_iter().map(
            |wrapped_points| {
                let point = wrapped_points.unwrap();

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
        print_log("[log] Voxelizer: Finish converting las to point list");

        print_log("[log] Voxelizer: Start creating VoxelPointCloud");
        let point_cloud = VoxelPointCloud::new(jpr_points, zoom_lv);
        print_log("[log] Voxelizer: Finish creating VoxelPointCloud");

        Self::voxelize(point_cloud)
    }


    #[cfg(feature = "las")]
    #[cfg(feature = "rayon")]
    pub fn voxelize_from_jpr_las<T>(las: T, jpr_origin: JprOrigin, zoom_lv: ZoomLv, rotate: bool) -> Vec<(TileIdx, VoxelMesh<f32>)>
        where T: BufRead + Seek + Send + Debug,
    {
        print_log("[log] Voxelizer: Start reading las file");
        let mut reader = Reader::new(las).unwrap();
        let points = reader.points().collect::<Vec<_>>();
        print_log("[log] Voxelizer: Finish reading las file");


        print_log("[log] Voxelizer: Start converting las to point list");
        let jpr_points = points.into_par_iter().map(
            |wrapped_points| {
                let point = wrapped_points.unwrap();

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
        print_log("[log] Voxelizer: Finish converting las to point list");

        print_log("[log] Voxelizer: Start creating VoxelPointCloud");
        let point_cloud = VoxelPointCloud::new(jpr_points, zoom_lv);
        print_log("[log] Voxelizer: Finish creating VoxelPointCloud");

        Self::voxelize(point_cloud)
    }

    /// Generate a list of `VoxelMesh` from `VoxelPointCloud`
    ///
    /// `VoxelPointCloud`から、`VoxelMesh`のリストを生成する
    #[cfg(feature = "rayon")]
    pub fn voxelize(point_cloud: VoxelPointCloud) -> Vec<(TileIdx, VoxelMesh<f32>)> {
        print_log("[log] Voxelizer: Start calculating min voxel coord");
        let min_voxel_coord = point_cloud.points.iter().fold(Coord::new([u32::MAX, u32::MAX, u32::MAX]), |min, (pixel_coord, _)| {
            Coord::new([min[0].min(pixel_coord[0]), min[1].min(pixel_coord[1]), min[2].min(pixel_coord[2])])
        });
        print_log("[log] Voxelizer: Finish calculating min voxel coord");

        print_log("[log] Voxelizer: Start creating voxel collection");
        let voxel_collection = if Params::TILING {
            print_log("[info] Voxelizer: Params::TILING is true");

            print_log("[log] Voxelizer: Start splitting point cloud");
            let split_points = point_cloud.split_by_tile();
            print_log("[log] Voxelizer: Finish splitting point cloud");

            split_points.into_par_iter().map(|(tile_idx, pixel_point_cloud)| {
                let voxel_collection = VoxelCollection::from_voxel_point_cloud(pixel_point_cloud, Params::THRESHOLD);

                (tile_idx, voxel_collection)
            }).collect::<Vec<_>>()
        } else {
            print_log("[info] Voxelizer: Params::TILING is false");

            let tile_idx = min_voxel_coord.fit() / 256_u32;

            vec![(tile_idx, VoxelCollection::from_voxel_point_cloud(point_cloud, Params::THRESHOLD))]
        };
        print_log("[log] Voxelizer: Finish creating voxel collection");

        let offset = match Params::OFFSET {
            Offset::MinTile => ((min_voxel_coord.fit::<2>() / 256_u32) * 256_u32).fit(),
            Offset::Pixel => Coord::new([min_voxel_coord[0], min_voxel_coord[1], 0]),
            Offset::Voxel => min_voxel_coord,
            _ => Coord::default(),
        };

        print_log("[log] Voxelizer: Start creating voxel mesh");
        let result = voxel_collection.into_par_iter().map(|(tile_idx, voxel_collection)| {
            let zoom_lv = voxel_collection.zoom_lv;

            let offset = if Params::OFFSET == Offset::Tile {
                (tile_idx * 256_u32).fit()
            } else {
                offset
            };

            let callback = |(coord, rgb): Point<u32>| -> Point<f32>{
                let voxel_size = {
                    let (_, lat) = pixel2ll((coord[0], coord[1]), zoom_lv);
                    pixel_resolution(lat, zoom_lv)
                };

                let coord = (coord - offset).as_() * voxel_size as f32;

                (coord, rgb)
            };

            let voxel_mesh = VoxelMesh::<u32>::from_voxel_collection(voxel_collection).batch_to_vertices(callback);

            (tile_idx, voxel_mesh)
        }).collect::<Vec<_>>();

        print_log("[log] Voxelizer: Finish creating voxel mesh");

        result
    }

    /// Generate a list of `VoxelMesh` from `VoxelPointCloud`
    ///
    /// `VoxelPointCloud`から、`VoxelMesh`のリストを生成する
    #[cfg(not(feature = "rayon"))]
    pub fn voxelize(point_cloud: VoxelPointCloud) -> Vec<(TileIdx, VoxelMesh<f32>)> {
        print_log("[log] Voxelizer: Start calculating min voxel coord");
        let min_voxel_coord = point_cloud.points.iter().fold(Coord::new([u32::MAX, u32::MAX, u32::MAX]), |min, (pixel_coord, _)| {
            Coord::new([min[0].min(pixel_coord[0]), min[1].min(pixel_coord[1]), min[2].min(pixel_coord[2])])
        });
        print_log("[log] Voxelizer: Finish calculating min voxel coord");

        print_log("[log] Voxelizer: Start creating voxel collection");
        let voxel_collection = if Params::TILING {
            print_log("[info] Voxelizer: Params::TILING is true");

            print_log("[log] Voxelizer: Start splitting point cloud");
            let split_points = point_cloud.split_by_tile();
            print_log("[log] Voxelizer: Finish splitting point cloud");

            split_points.into_iter().map(|(tile_idx, pixel_point_cloud)| {
                let voxel_collection = VoxelCollection::from_voxel_point_cloud(pixel_point_cloud, Params::THRESHOLD);

                (tile_idx, voxel_collection)
            }).collect::<Vec<_>>()
        } else {
            print_log("[info] Voxelizer: Params::TILING is false");

            let tile_idx = min_voxel_coord.fit() / 256_u32;

            vec![(tile_idx, VoxelCollection::from_voxel_point_cloud(point_cloud, Params::THRESHOLD))]
        };
        print_log("[log] Voxelizer: Finish creating voxel collection");

        let offset = match Params::OFFSET {
            Offset::MinTile => ((min_voxel_coord.fit::<2>() / 256_u32) * 256_u32).fit(),
            Offset::Pixel => Coord::new([min_voxel_coord[0], min_voxel_coord[1], 0]),
            Offset::Voxel => min_voxel_coord,
            _ => Coord::default(),
        };

        print_log("[log] Voxelizer: Start creating voxel mesh");
        let result = voxel_collection.into_iter().map(|(tile_idx, voxel_collection)| {
            let zoom_lv = voxel_collection.zoom_lv;

            let offset = if Params::OFFSET == Offset::Tile {
                (tile_idx * 256_u32).fit()
            } else {
                offset
            };

            let callback = |(coord, rgb): Point<u32>| -> Point<f32>{
                let voxel_size = {
                    let (_, lat) = pixel2ll((coord[0], coord[1]), zoom_lv);
                    pixel_resolution(lat, zoom_lv)
                };

                let coord = (coord - offset).as_() * voxel_size as f32;

                (coord, rgb)
            };

            let voxel_mesh = VoxelMesh::<u32>::from_voxel_collection(voxel_collection).batch_to_vertices(callback);

            (tile_idx, voxel_mesh)
        }).collect::<Vec<_>>();

        print_log("[log] Voxelizer: Finish creating voxel mesh");

        result
    }
}
