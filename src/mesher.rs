use std::marker::PhantomData;

use dashmap::DashMap;
use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use num::cast::AsPrimitive;

use crate::collection::VoxelCollection;
use crate::element::{Color, Int, Point, Point3D, UInt};

type VertexIds = Vec<usize>;

#[derive(Default, Debug, Clone)]
pub struct VoxelMesh<P: Int, C: UInt> {
    pub(crate) bounds: (Point3D<P>, Point3D<P>),
    pub(crate) offset: Point3D<P>,
    pub(crate) vertices: IndexSet<Point3D<P>>,
    pub(crate) faces: DashMap<Color<C>, VertexIds, FxBuildHasher>,
}

pub trait Mesher<P, W, C, VC>
where
    P: Int,
    W: UInt,
    C: UInt,
    VC: VoxelCollection<P, W, C>,
{
    fn new(voxel_collection: VC) -> Self;
    fn meshing(self) -> VoxelMesh<P, C>;
}

pub struct SimpleMesher<P, W, C, VC>
where
    P: Int,
    W: UInt,
    C: UInt,
    VC: VoxelCollection<P, W, C>,
{
    _phantom: PhantomData<W>,
    voxel_collection: VC,
    mesh: VoxelMesh<P, C>,
}

impl<P, W, C, VC> Mesher<P, W, C, VC> for SimpleMesher<P, W, C, VC>
where
    P: Int + AsPrimitive<i32>,
    W: UInt + AsPrimitive<C>,
    C: UInt + AsPrimitive<W>,
    VC: VoxelCollection<P, W, C>,
    i32: AsPrimitive<P>,
{
    fn new(mut voxel_collection: VC) -> Self {
        let mesh = VoxelMesh {
            bounds: voxel_collection.get_bounds(),
            offset: voxel_collection.get_offset(),
            vertices: IndexSet::new(),
            faces: DashMap::with_hasher(FxBuildHasher::default()),
        };


        SimpleMesher {
            _phantom: PhantomData,
            voxel_collection,
            mesh,
        }
    }
    fn meshing(mut self) -> VoxelMesh<P, C> {
        let exist_check = |point: Option<Point3D<P>>| {
            if let Some(point) = point {
                self.voxel_collection.has(&point)
            } else {
                false
            }
        };

        self.voxel_collection.to_points().into_iter().for_each(|(point, color)| {
            // todo exist_checkを使った無駄な面の排除
            
            let faces = [
            // [(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 1, 1), (1, 0, 1), (1, 0, 0)] // 右
            // [(0, 0, 1), (0, 1, 1), (0, 1, 0), (0, 1, 0), (0, 0, 0), (0, 0, 1)] // 左   
            // [(0, 1, 1), (1, 1, 1), (1, 1, 0), (1, 1, 0), (0, 1, 0), (0, 1, 1)] // 下
            // [(0, 0, 0), (1, 0, 0), (1, 0, 1), (1, 0, 1), (0, 0, 1), (0, 0, 0)] // 上
            // [(0, 0, 1), (1, 0, 1), (1, 1, 1), (1, 1, 1), (0, 1, 1), (0, 0, 1)] // 前
            // [(0, 1, 0), (1, 1, 0), (1, 0, 0), (1, 0, 0), (0, 0, 0), (0, 1, 0)] // 後
            ([(1_i32, 0, 0), (1, 1, 0), (1, 1, 1), (1, 1, 1), (1, 0, 1), (1, 0, 0)] ,exist_check(point.right())),
            ([(0, 0, 1), (0, 1, 1), (0, 1, 0), (0, 1, 0), (0, 0, 0), (0, 0, 1)] ,exist_check(point.left())), 
            ([(0, 1, 1), (1, 1, 1), (1, 1, 0), (1, 1, 0), (0, 1, 0), (0, 1, 1)] ,exist_check(point.top())),
            ([(0, 0, 0), (1, 0, 0), (1, 0, 1), (1, 0, 1), (0, 0, 1), (0, 0, 0)] ,exist_check(point.bottom())),
            ([(0, 0, 1), (1, 0, 1), (1, 1, 1), (1, 1, 1), (0, 1, 1), (0, 0, 1)] ,exist_check(point.front())),
            ([(0, 1, 0), (1, 1, 0), (1, 0, 0), (1, 0, 0), (0, 0, 0), (0, 1, 0)] ,exist_check(point.back())), 
            ].into_iter().flat_map(|(vertexes, _)| {
                vertexes.into_iter().map(|(dx, dy, dz)| {
                    point + Point3D::new([dx, dy, dz]).as_()
                })
            }).collect::<Vec<_>>();

            if faces.is_empty() {
                return;
            }

            let mut vertex_indices = faces.into_iter().map(|point| self.mesh.vertices.insert_full(point).0);

            self.mesh.faces.entry(color).and_modify(|vertexes| vertexes.extend(&mut vertex_indices)).or_insert(vertex_indices.collect());
        });

        // メッシュを貼った分
        self.mesh.bounds.1 +=P::one();

        self.mesh
    }
}
