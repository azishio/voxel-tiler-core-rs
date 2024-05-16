use std::collections::HashMap;
use std::hash::Hash;

use coordinate_transformer::pixel_ll::ZoomLv;
use fxhash::FxBuildHasher;
use num::Num;
use vec_x::VecX;

type Coord<T: Num> = VecX<T, 3>;
pub type RGB = VecX<u8, 3>;
pub type Point<T: Num> = (Coord<T>, RGB);

type SumRGB = VecX<u32, 3>;
type TileIdx = VecX<u32, 2>;

struct PointCloud<T: Num> {
    points: Vec<Point<T>>,
    voxel_size: f32,
    zoom_lv: ZoomLv,
}

impl<T: Num + Copy> PointCloud<T> {
    pub fn new(points: Vec<Point<T>>, voxel_size: f32, zoom_lv: ZoomLv) -> Self {
        Self {
            points,
            voxel_size,
            zoom_lv,
        }
    }

    pub fn empty() -> Self {
        Self {
            points: Vec::new(),
            voxel_size: 0.,
            zoom_lv: ZoomLv::Lv0,
        }
    }

    pub fn coordinate_transform<U: Num>(self, f: fn(Coord<T>) -> Coord<U>) -> PointCloud<U> {
        let Self { points, voxel_size, zoom_lv } = self;

        let points = points.into_iter().map(|(coord, rgb)| (f(coord), rgb)).collect::<Vec<_>>();

        PointCloud {
            points,
            voxel_size,
            zoom_lv,
        }
    }
}

impl PointCloud<u32> {
    pub fn split_by_tile(self) -> Vec<(TileIdx, PointCloud<u32>)> {
        let mut tiled_points = HashMap::<TileIdx, PointCloud<u32>, FxBuildHasher>::with_hasher(Default::default());

        self.points.into_iter().for_each(|(pixel_coord, rgb)| {
            let tile_idx = {
                let x = pixel_coord[0];
                let y = pixel_coord[1];
                let tile_x = x / 256;
                let tile_y = y / 256;
                TileIdx::new([tile_x, tile_y])
            };

            tiled_points.entry(tile_idx).or_insert(PointCloud::new(Vec::new(), self.voxel_size, self.zoom_lv)).points.push((pixel_coord, rgb));
        });

        tiled_points.into_iter().collect::<Vec<_>>()
    }
}

pub struct VoxelCollection<T: Num> {
    pub(crate) voxels: Vec<Point<T>>,
    pub(crate) voxel_size: f32,
    pub(crate) zoom_lv: ZoomLv,
}

impl<T: Num + Eq + Hash> VoxelCollection<T> {
    pub fn new(
        voxels: Vec<Point<T>>,
        voxel_size: f32,
        zoom_lv: ZoomLv,
    ) -> Self {
        Self {
            voxels,
            voxel_size,
            zoom_lv,
        }
    }

    pub fn empty() -> Self {
        Self {
            voxels: Vec::new(),
            voxel_size: 0.,
            zoom_lv: ZoomLv::Lv0,
        }
    }

    pub fn from_point_cloud(point_cloud: PointCloud<T>) -> Self {
        let PointCloud {
            points,
            voxel_size,
            zoom_lv,
        } = point_cloud;


        let mut voxel_map = HashMap::<Coord<T>, (u32, SumRGB), FxBuildHasher>::with_hasher(Default::default());

        points.into_iter().for_each(|(pixel_coord, rgb)| {
            let rgb = SumRGB::new([rgb[0] as u32, rgb[1] as u32, rgb[2] as u32]);

            voxel_map.entry(pixel_coord).and_modify(|(count, sum_rgb)| {
                *sum_rgb += rgb;
                *count += 1;
            }).or_insert((1, Coord::new([0, 0, 0])));
        });

        let voxels = voxel_map.into_iter().map(|(pixel_coord, (count, sum_rgb))| {
            let rgb = RGB::new([
                (sum_rgb[0] / count) as u8,
                (sum_rgb[1] / count) as u8,
                (sum_rgb[2] / count) as u8,
            ]);
            (pixel_coord, rgb)
        }).collect::<Vec<_>>();

        Self {
            voxels,
            voxel_size,
            zoom_lv,
        }
    }

    pub fn coordinate_transform<U: Num>(self, f: fn(Coord<T>) -> Coord<U>) -> VoxelCollection<U> {
        let Self { voxels, voxel_size, zoom_lv } = self;

        let voxels = voxels.into_iter().map(|(coord, rgb)| (f(coord), rgb)).collect::<Vec<_>>();

        VoxelCollection {
            voxels,
            voxel_size,
            zoom_lv,
        }
    }
}
