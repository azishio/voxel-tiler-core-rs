use std::collections::HashMap;

use coordinate_transformer::pixel_ll::ZoomLv;
use fxhash::FxBuildHasher;
use vec_x::VecX;

type PixelCoord = VecX<u32, 3>;
type RGB = VecX<u8, 3>;
type Point = (PixelCoord, RGB);

type LooclPixelCoord = PixelCoord;

// 一つのボクセル内に頂点が何個存在することを想定するかを考える
type SumRGB = VecX<u32, 3>;
type TileIdx = VecX<u32, 2>;

struct PointCloud {
    points: Vec<Point>,
    voxel_size: u32,
    zoom_lv: ZoomLv,
}

impl PointCloud {
    pub fn new(points: Vec<Point>, voxel_size: u32, zoom_lv: ZoomLv) -> Self {
        Self {
            points,
            voxel_size,
            zoom_lv,
        }
    }

    pub fn empty() -> Self {
        Self {
            points: Vec::new(),
            voxel_size: 0,
            zoom_lv: ZoomLv::Lv0,
        }
    }


    pub fn split_by_tile(self) -> Vec<(TileIdx, PointCloud)> {
        let mut tiled_points = HashMap::<TileIdx, PointCloud, FxBuildHasher>::with_hasher(Default::default());

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

pub struct VoxelCollection {
    voxels: Vec<Point>,
    voxel_size: u32,
    zoom_lv: ZoomLv,
}

impl VoxelCollection {
    pub fn new(
        voxels: Vec<Point>,
        voxel_size: u32,
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
            voxel_size: 0,
            zoom_lv: ZoomLv::Lv0,
        }
    }

    pub fn from_point_cloud(point_cloud: PointCloud) -> Self {
        let PointCloud {
            points,
            voxel_size,
            zoom_lv,
        } = point_cloud;


        let mut voxel_map = HashMap::<PixelCoord, (u32, SumRGB), FxBuildHasher>::with_hasher(Default::default());

        points.into_iter().for_each(|(pixel_coord, rgb)| {
            let rgb = SumRGB::new([rgb[0] as u32, rgb[1] as u32, rgb[2] as u32]);

            voxel_map.entry(pixel_coord).and_modify(|(count, sum_rgb)| {
                *sum_rgb += rgb;
                *count += 1;
            }).or_insert((1, PixelCoord::new([0, 0, 0])));
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
}
