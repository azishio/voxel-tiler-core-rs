use std::fmt::Debug;
use std::io::{BufRead, Seek};
use std::marker::PhantomData;

use coordinate_transformer::{jpr2ll, JprOrigin, ll2pixel, pixel2ll, pixel_resolution, ZoomLv};
#[cfg(feature = "las")]
use las::{Color, Read, Reader};

use crate::{Coord, default_params, Offset, PixelPointCloud, Point, RGB, TileIdx, VoxelCollection, VoxelizerParams, VoxelMesh};

pub struct VoxelModel {
    pub voxel_mesh: VoxelMesh<f32>,
    pub min_voxel_coord: Coord<u32>,
}


pub struct Voxelizer<Params: VoxelizerParams = default_params::Fit> {
    _param: PhantomData<Params>,
}

impl<Params: VoxelizerParams> Voxelizer<Params> {
    #[cfg(feature = "las")]
    pub fn voxelize_from_jpr_las<T>(las: T, jpr_origin: JprOrigin, zoom_lv: ZoomLv, rotate: bool) -> Vec<(TileIdx, VoxelMesh<f32>)>
        where T: BufRead + Seek + Send + Debug,
    {
        let mut reader = Reader::new(las).unwrap();

        let points = reader.points().collect::<Vec<_>>();

        let jpr_points = points.into_iter().map(|wrapped_points| {
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

        let point_cloud = PixelPointCloud::new(jpr_points, zoom_lv);

        Self::voxelize(point_cloud)
    }

    pub fn voxelize(point_cloud: PixelPointCloud) -> Vec<(TileIdx, VoxelMesh<f32>)> {
        let min_voxel_coord = point_cloud.points.iter().fold(Coord::new([u32::MAX, u32::MAX, u32::MAX]), |min, (pixel_coord, _)| {
            Coord::new([min[0].min(pixel_coord[0]), min[1].min(pixel_coord[1]), min[2].min(pixel_coord[2])])
        });

        let voxel_collection = if Params::TILING {
            VoxelCollection::from_pixel_point_cloud_with_tiling(point_cloud, Params::THRESHOLD)
        } else {
            let tile_idx = min_voxel_coord.fit() / 256_u32;

            vec![(tile_idx, VoxelCollection::from_pixel_point_cloud(point_cloud, Params::THRESHOLD))]
        };

        let offset = match Params::OFFSET {
            Offset::MinTile => ((min_voxel_coord.fit::<2>() / 256_u32) * 256_u32).fit(),
            Offset::Pixel => Coord::new([min_voxel_coord[0], min_voxel_coord[1], 0]),
            Offset::Voxel => min_voxel_coord,
            _ => Coord::default(),
        };

        voxel_collection.into_iter().map(|(tile_idx, voxel_collection)| {
            let zoom_lv = voxel_collection.zoom_lv;

            let offset = if Params::OFFSET == Offset::Tile {
                (tile_idx * 256_u32).fit()
            } else {
                offset
            };

            let callback = |v: Coord<u32>| -> Coord<f32>{
                let voxel_size = {
                    let (_, lat) = pixel2ll((v[0], v[1]), zoom_lv);
                    pixel_resolution(lat, zoom_lv)
                };

                (v - offset).as_() * voxel_size as f32
            };

            let voxel_mesh = VoxelMesh::<u32>::from_voxel_collection(voxel_collection).coordinate_transform(callback);

            (tile_idx, voxel_mesh)
        }).collect::<Vec<_>>()
    }
}
