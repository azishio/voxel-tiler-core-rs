use dashmap::DashMap;
use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use meshopt::{simplify_decoder, SimplifyOptions};
use num::cast::AsPrimitive;

use crate::element::{Color, Int, Point3D, UInt};

/// メッシュが貼られたボクセルを表す構造体です。
#[derive(Default, Debug, Clone)]
pub struct VoxelMesh<P: Int, C: UInt> {
    pub(crate) bounds: (Point3D<P>, Point3D<P>),
    pub(crate) offset: Point3D<P>,
    pub(crate) points: IndexSet<Point3D<P>, FxBuildHasher>,
    pub(crate) faces: DashMap<Color<C>, Vec<usize>, FxBuildHasher>,
    pub(crate) resolution: f64,
}

impl<P: Int, C: UInt> VoxelMesh<P, C>
where
    P: Int + AsPrimitive<f32>,
    C: UInt + AsPrimitive<f32>,
    f32: AsPrimitive<P>,
{
    ///[`simplify_decoder`]を使用してメッシュを簡略化します。
    pub fn simplify(self) -> Self
    {
        let VoxelMesh { points, faces, bounds, offset, resolution, .. } = self;

        let point_f32: Vec<[f32; 3]> = points.iter()
            .map(|point| point.as_::<f32>().data)
            .collect();

        let mut new_points = IndexSet::<Point3D<P>, FxBuildHasher>::with_hasher(Default::default());

        let simplified_points = faces.into_iter().map(|(color, indices)| {
            let indices: Vec<u32> = indices.into_iter()
                .filter_map(|i| i.try_into().ok()).collect();

            let new_indices = simplify_decoder(&indices, &point_f32, 0, 0.05, SimplifyOptions::all(), None)
                .into_iter().map(|i| {
                new_points.insert_full(points[i as usize]).0
            }).collect::<Vec<_>>();

            (color, new_indices)
        }).collect::<DashMap<_, _, _>>();


        VoxelMesh {
            bounds,
            offset,
            points: new_points,
            faces: simplified_points,
            resolution,
        }
    }
}
