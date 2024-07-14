use bitflags::bitflags;
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use meshopt::{simplify_decoder, SimplifyOptions};
use num::cast::AsPrimitive;

use crate::collection::VoxelCollection;
use crate::element::{Color, Int, Point, Point3D, UInt};

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

bitflags! {
    /// ボクセルの有効な面を表すビットフラグです。
    /// このフラグが立っている面にのみメッシュを生成します。
    /// 具体的なフラグの使用方法は[`bitflags`]のドキュメントを参照してください。
        pub struct ValidSide: u8 {
        /// 上面
            const TOP = 0b00000001;

        /// 下面
            const BOTTOM = 0b00000010;

        /// 左面
            const LEFT = 0b00000100;

        /// 右面
            const RIGHT = 0b00001000;

        /// 前面
            const FRONT = 0b00010000;

        /// 後面
            const BACK = 0b00100000;

        /// 境界
            const BORDER = 0b01000000;
        }
    }



/// ボクセルメッシュを生成するための構造体です。
pub struct Mesher;

impl Mesher
{
    /// ボクセルメッシュを生成します。
    pub fn meshing<P, W, C, VCF>(mut vc: VCF, valid_side: ValidSide) -> VoxelMesh<P, C>
    where
        P: Int + AsPrimitive<i32>,
        W: UInt + AsPrimitive<C>,
        C: UInt + AsPrimitive<W>,
        VCF: VoxelCollection<P, W, C>,
        i32: AsPrimitive<P>,
    {
        let mut mesh = VoxelMesh {
            bounds: vc.get_bounds(),
            offset: vc.get_offset(),
            resolution: vc.get_resolution(),
            ..Default::default()
        };

        // ボクセルのAABBから頂点のAABBにったため
        mesh.bounds.1 += P::one();


        let is_required = |neighbor: Option<Point3D<P>>| {
            if let Some(neighbor) = neighbor {
                // 隣接ボクセルが存在する場合
                if vc.has(&neighbor) {
                    return false;
                }
            };
            true
        };

        let on_border = |point: Point3D<P>| -> bool{
            let (min, max) = mesh.bounds;

            point[0] == min[0] || point[0] == max[0] ||
                point[1] == min[1] || point[1] == max[1] ||
                point[2] == min[2] || point[2] == max[2]
        };

        vc.to_points().into_iter().for_each(|(point, color)| {
            let unit_faces = [
                (valid_side.contains(ValidSide::LEFT), [(0, 0, 0), (0, 0, 1), (0, 1, 1), (0, 1, 1), (0, 1, 0), (0, 0, 0)], is_required(point.left())),
                (valid_side.contains(ValidSide::RIGHT), [(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 1, 1), (1, 0, 1), (1, 0, 0)], is_required(point.right())),
                (valid_side.contains(ValidSide::BOTTOM), [(0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 1, 0), (1, 0, 0), (0, 0, 0)], is_required(point.bottom())),
                (valid_side.contains(ValidSide::TOP), [(0, 0, 1), (1, 0, 1), (1, 1, 1), (1, 1, 1), (0, 1, 1), (0, 0, 1)], is_required(point.top())),
                (valid_side.contains(ValidSide::BACK), [(0, 0, 0), (1, 0, 0), (1, 0, 1), (1, 0, 1), (0, 0, 1), (0, 0, 0)], is_required(point.back())),
                (valid_side.contains(ValidSide::FRONT), [(1, 1, 1), (1, 1, 0), (0, 1, 0), (0, 1, 0), (0, 1, 1), (1, 1, 1)], is_required(point.front())),
            ].into_iter()
                .filter(|&(valid, _, required)| valid && required)
                .filter_map(|(_, delta, _)| {
                    let vertices = delta.into_iter().map(|(dx, dy, dz)| {
                        point + Point3D::new([dx, dy, dz]).as_()
                    });

                    if !valid_side.contains(ValidSide::BORDER) {
                        if vertices.clone().any(on_border) {
                            return None;
                        }
                    }

                    Some(vertices)
                }).flatten().collect::<Vec<_>>();

            if unit_faces.is_empty() {
                return;
            }

            let mut vertex_indices = unit_faces.into_iter().map(|point| mesh.points.insert_full(point).0);

            mesh.faces.entry(color).and_modify(|t| t.extend(&mut vertex_indices)).or_insert(vertex_indices.collect());
        });

        mesh
    }
}
