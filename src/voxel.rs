use std::collections::HashMap;

use coordinate_transformer::pixel_ll::ZoomLv;
use fxhash::FxBuildHasher;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use vec_x::VecX;

pub type Coord<T> = VecX<T, 3>;
pub type RGB = VecX<u8, 3>;
pub type Point<T> = (Coord<T>, RGB);

pub type TileIdx = VecX<u32, 2>;

/// Structure representing a point cloud expressed in pixel coordinates
///
/// ピクセル座標で表された点群を表す構造体
pub struct VoxelPointCloud {
    /// Point cloud
    ///
    /// 点群
    pub points: Vec<Point<u32>>,

    /// Zoom level of pixel coordinates of point cloud
    ///
    /// 点群のピクセル座標のズームレベル
    pub zoom_lv: ZoomLv,
}

impl VoxelPointCloud {
    /// Generate a new `VoxelPointCloud`
    ///
    /// 新しい`VoxelPointCloud`を生成します。
    pub fn new(points: Vec<Point<u32>>, zoom_lv: ZoomLv) -> Self {
        Self {
            points,
            zoom_lv,
        }
    }

    /// Generate an empty `VoxelPointCloud`.
    /// It is not practically meaningful because it does not contain data, but it is useful when writing temporary code.
    ///
    /// 空の`VoxelPointCloud`を生成します。
    /// データを内包しないため実用上の意味はありませんが、仮のコードを書く際に便利です。
    pub fn empty() -> Self {
        Self {
            points: Vec::new(),
            zoom_lv: ZoomLv::Lv0,
        }
    }

    /// Returns the point cloud divided by tile.
    ///
    /// タイルごとに分割した点群を返します。
    pub fn split_by_tile(self) -> Vec<(TileIdx, VoxelPointCloud)> {
        let mut tiled_points = HashMap::<TileIdx, VoxelPointCloud, FxBuildHasher>::with_hasher(Default::default());

        self.points.into_iter().for_each(|(pixel_coord, rgb)| {
            let tile_idx = {
                let x = pixel_coord[0];
                let y = pixel_coord[1];
                let tile_x = x / 256;
                let tile_y = y / 256;
                TileIdx::new([tile_x, tile_y])
            };

            tiled_points.entry(tile_idx).or_insert(VoxelPointCloud::new(Vec::new(), self.zoom_lv)).points.push((pixel_coord, rgb));
        });

        tiled_points.into_iter().collect::<Vec<_>>()
    }
}

/// Structure representing a group of voxels.
///
/// ボクセル群を表す構造体。
///
/// A voxel is a cube with a side length of one pixel in pixel coordinates.
/// Resolution depends on latitude and zoom level in pixel coordinates.
///
/// ボクセルとは、ピクセル座標における1ピクセル分の辺長を持つ立方体のことです。
/// 分解能は緯度とピクセル座標のズームレベルに依存します。
pub struct VoxelCollection {
    /// A list of unique coordinate and RGB value pairs.
    /// The coordinate value held by Point must be unique.
    ///
    /// 一意な座標とRGB値のペアのリスト。
    /// Pointが持つ座標値はユニークな値である必要があります。
    pub voxels: Vec<Point<u32>>,

    /// Zoom level of pixel coordinates of voxel group.
    ///
    /// ボクセル群のピクセル座標のズームレベル。
    pub zoom_lv: ZoomLv,
}

impl VoxelCollection {
    /// Generate a new `VoxelCollection`.
    ///
    /// 新しい`VoxelCollection`を生成します。
    pub fn new(
        voxels: Vec<Point<u32>>,
        zoom_lv: ZoomLv,
    ) -> Self {
        Self {
            voxels,
            zoom_lv,
        }
    }

    /// Generate an empty `VoxelCollection`.
    /// It is not practically meaningful because it does not contain data, but it is useful when writing temporary code.
    ///
    /// 空の`VoxelCollection`を生成します。
    /// データを内包しないため実用上の意味はありませんが、仮のコードを書く際に便利です。
    pub fn empty() -> Self {
        Self {
            voxels: Vec::new(),
            zoom_lv: ZoomLv::Lv0,
        }
    }

    /// Generate a `VoxelCollection` from a `VoxelPointCloud`.
    ///
    /// `VoxelPointCloud`から`VoxelCollection`を生成します。
    pub fn from_voxel_point_cloud(voxel_point_cloud: VoxelPointCloud, threshold: usize) -> Self {
        type SumRGB = VecX<usize, 3>;

        let VoxelPointCloud { points, zoom_lv } = voxel_point_cloud;

        let mut voxel_map = HashMap::<Coord<u32>, (usize, SumRGB), FxBuildHasher>::with_hasher(Default::default());

        points.into_iter().for_each(|(pixel_coord, rgb)| {
            let rgb = SumRGB::new([rgb[0] as usize, rgb[1] as usize, rgb[2] as usize]);

            voxel_map.entry(pixel_coord).and_modify(|(count, sum_rgb)| {
                *sum_rgb += rgb;
                *count += 1;
            }).or_insert((0, Coord::new([0, 0, 0])));
        });

        let voxels = {
            let f = |(pixel_coord, (count, sum_rgb)): (Coord<u32>, (usize, SumRGB))| {
                if count == 0 {
                    return None;
                }
                if count < threshold {
                    return None;
                }

                let rgb = RGB::new([
                    (sum_rgb[0] / count) as u8,
                    (sum_rgb[1] / count) as u8,
                    (sum_rgb[2] / count) as u8,
                ]);
                Some((pixel_coord, rgb))
            };

            if cfg!(feature = "rayon") {
                voxel_map.into_iter().filter_map(f).collect::<Vec<_>>()
            } else {
                voxel_map.into_iter().filter_map(f).collect::<Vec<_>>()
            }
        };


        Self { voxels, zoom_lv }
    }
}
