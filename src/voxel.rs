use std::collections::HashMap;

use coordinate_transformer::pixel_ll::ZoomLv;
use fxhash::FxBuildHasher;
use vec_x::VecX;

pub type Coord<T> = VecX<T, 3>;
pub type RGB = VecX<u8, 3>;
pub type Point<T> = (Coord<T>, RGB);

type SumRGB = VecX<usize, 3>;
pub type TileIdx = VecX<u32, 2>;

pub struct PixelPointCloud {
    pub points: Vec<Point<u32>>,
    pub zoom_lv: ZoomLv,
}

impl PixelPointCloud {
    pub fn new(points: Vec<Point<u32>>, zoom_lv: ZoomLv) -> Self {
        Self {
            points,
            zoom_lv,
        }
    }

    pub fn empty() -> Self {
        Self {
            points: Vec::new(),
            zoom_lv: ZoomLv::Lv0,
        }
    }

    pub(crate) fn split_by_tile(self) -> Vec<(TileIdx, PixelPointCloud)> {
        let mut tiled_points = HashMap::<TileIdx, PixelPointCloud, FxBuildHasher>::with_hasher(Default::default());

        self.points.into_iter().for_each(|(pixel_coord, rgb)| {
            let tile_idx = {
                let x = pixel_coord[0];
                let y = pixel_coord[1];
                let tile_x = x / 256;
                let tile_y = y / 256;
                TileIdx::new([tile_x, tile_y])
            };

            tiled_points.entry(tile_idx).or_insert(PixelPointCloud::new(Vec::new(), self.zoom_lv)).points.push((pixel_coord, rgb));
        });

        tiled_points.into_iter().collect::<Vec<_>>()
    }
}

pub struct VoxelCollection {
    pub(crate) voxels: Vec<Point<u32>>,
    pub(crate) zoom_lv: ZoomLv,
}

impl VoxelCollection {
    pub fn new(
        voxels: Vec<Point<u32>>,
        zoom_lv: ZoomLv,
    ) -> Self {
        Self {
            voxels,
            zoom_lv,
        }
    }

    pub fn empty() -> Self {
        Self {
            voxels: Vec::new(),
            zoom_lv: ZoomLv::Lv0,
        }
    }

    pub fn from_pixel_point_cloud(point_cloud: PixelPointCloud) -> Vec<(TileIdx, Self)> {
        let split_points = point_cloud.split_by_tile();

        split_points.into_iter().map(|(tile_idx, pixel_point_cloud)| {
            let PixelPointCloud { points, zoom_lv } = pixel_point_cloud;

            let mut voxel_map = HashMap::<Coord<u32>, (usize, SumRGB), FxBuildHasher>::with_hasher(Default::default());

            points.into_iter().for_each(|(pixel_coord, rgb)| {
                let rgb = SumRGB::new([rgb[0] as usize, rgb[1] as usize, rgb[2] as usize]);

                voxel_map.entry(pixel_coord).and_modify(|(count, sum_rgb)| {
                    *sum_rgb += rgb;
                    *count += 1;
                }).or_insert((0, Coord::new([0, 0, 0])));
            });

            let voxels = voxel_map.into_iter().filter_map(|(pixel_coord, (count, sum_rgb))| {
                if count == 0 {
                    return None;
                }

                let rgb = RGB::new([
                    (sum_rgb[0] / count) as u8,
                    (sum_rgb[1] / count) as u8,
                    (sum_rgb[2] / count) as u8,
                ]);
                Some((pixel_coord, rgb))
            }).collect::<Vec<_>>();

            (tile_idx, Self { voxels, zoom_lv })
        }).collect::<Vec<_>>()
    }
}
