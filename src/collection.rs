use std::hash::BuildHasher;
use std::marker::PhantomData;

use dashmap::DashMap;
use num::traits::AsPrimitive;

use crate::collection::private::PrivateVoxelCollectionMethod;
use crate::element::{Color, Int, Number, Point2D, Point3D, UInt, Voxel};

mod private {
    use num::cast::AsPrimitive;

    use crate::element::{Color, Number, Point3D, UInt, Voxel};

    pub trait PrivateVoxelCollectionMethod<P: Number, W: UInt, C: UInt>
    where
        Self: Sized,
    {
        fn get_inner_bounds(&self) -> Option<(Point3D<P>, Point3D<P>)>;
        fn set_inner_bounds(&mut self, bounds: (Point3D<P>, Point3D<P>));
        fn calc_bounds(points: &Vec<(Point3D<P>, Voxel<C, W>)>) -> (Point3D<P>, Point3D<P>) {
            if points.is_empty() {
                return (Point3D::default(), Point3D::default());
            }

            let max = Point3D::new([
                points.iter().map(|(p, _v)| p[0]).max().unwrap(),
                points.iter().map(|(p, _v)| p[1]).max().unwrap(),
                points.iter().map(|(p, _v)| p[2]).max().unwrap(),
            ]);

            let min = Point3D::new([
                points.iter().map(|(p, _v)| p[0]).min().unwrap(),
                points.iter().map(|(p, _v)| p[1]).min().unwrap(),
                points.iter().map(|(p, _v)| p[2]).min().unwrap(),
            ]);

            (min, max)
        }

        fn calc_bounds_from_2((min1, max1): (Point3D<P>, Point3D<P>), (min2, max2): (Point3D<P>, Point3D<P>)) -> (Point3D<P>, Point3D<P>) {
            let min = min1.batch_with(min2, |a, b| a.min(b));
            let max = max1.batch_with(max2, |a, b| a.max(b));

            (min, max)
        }

        // 重みを考慮して色を加算する
        fn add_color_with_weight_check(current_voxel: &mut Voxel<C, W>, voxel: Voxel<C, W>)
        where
            C: AsPrimitive<W>,
            W: AsPrimitive<C>,
        {
            if current_voxel.weight == W::max_value() {
                return;
            }

            if current_voxel.weight.checked_add(&voxel.weight).is_none() {
                let weight = W::max_value() - current_voxel.weight;
                current_voxel.weight += weight;
                current_voxel.color += voxel.color * Color::from(weight).as_::<C>();
            } else {
                current_voxel.weight += voxel.weight;
                current_voxel.color += voxel.color * Color::from(voxel.weight).as_::<C>();
            }
        }
    }
}

pub trait VoxelCollection<P, W, C>: PrivateVoxelCollectionMethod<P, W, C> + Default
where
    P: Number,
    W: UInt,
    C: UInt,
    Self: Sized,
{
    
    fn from_points(points: Vec<(Point3D<P>, Color<C>)>) -> Self {
        Self::from_voxels(points.into_iter().map(|(point, color)| {
            (point, Voxel::new(color))
        }).collect())
    }

    fn from_voxels(voxels: Vec<(Point3D<P>, Voxel<C, W>)>) -> Self {
        Self::with_offset(voxels, Point3D::default())
    }

    fn with_bounds(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: (Point3D<P>, Point3D<P>)) -> Self {
        Self::with_bounds_and_offset(voxels, Some(bounds), Point3D::default())
    }
    fn with_offset(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, offset: Point3D<P>) -> Self {
        Self::with_bounds_and_offset(voxels, None, offset)
    }

    fn with_bounds_and_offset(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>) -> Self;

    fn has_bounds(&self) -> bool;

    fn get_bounds(&mut self) -> (Point3D<P>, Point3D<P>) {
        self.get_inner_bounds().unwrap_or_else(|| {
            let (min, max) = Self::calc_bounds(&self.to_vec());
            let bounds = (min + self.get_offset(), max + self.get_offset());

            self.set_inner_bounds(bounds);

            bounds
        })
    }
    
    fn get_offset(&self) -> Point3D<P>;
    fn set_offset(&mut self, offset: Point3D<P>);

    fn offset_to_min(&mut self) {
        let current_offset = self.get_offset();

        let (min, _max) = self.get_bounds();

        if current_offset != min {
            let offset = min - current_offset;
            self.set_offset(offset);
        }
    }


    fn to_vec(&self) -> Vec<(Point3D<P>, Voxel<C, W>)>;
    fn into_vec(self) -> Vec<(Point3D<P>, Voxel<C, W>)>;
    fn to_vec_with_offset(&self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        // TODO これを使うべき箇所を探して治す
        let offset = self.get_offset();
        self.to_vec().into_iter().map(|(point, voxel)| {
            (point + offset, voxel)
        }).collect()
    }

    fn into_vec_with_offset(self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        let offset = self.get_offset();
        self.into_vec().into_iter().map(|(point, voxel)| {
            (point + offset, voxel)
        }).collect()
    }

    fn to_points(&self) -> Vec<(Point3D<P>, Color<C>)>
    where
        C: AsPrimitive<W>,
        W: AsPrimitive<C>,
    {
        self.to_vec().into_iter().map(|(point, voxel)| {
            let average_color = voxel.color / Color::from(voxel.weight).as_::<C>();
            (point, average_color)
        }).collect()
    }

    fn into_points(self) -> Vec<(Point3D<P>, Color<C>)>
    where
        C: AsPrimitive<W>,
        W: AsPrimitive<C>,
    {
        self.into_vec().into_iter().map(|(point, voxel)| {
            let average_color = voxel.color / Color::from(voxel.weight).as_::<C>();
            (point, average_color)
        }).collect()
    }

    fn insert<T: VoxelCollection<P, W, C>>(&mut self, pc: T) {
        pc.into_vec().into_iter().for_each(|(point, voxel)| {
            self.insert_one(point, voxel)
        });
    }

    fn insert_one(&mut self, point: Point3D<P>, voxel: Voxel<C, W>);

    fn merge<T: VoxelCollection<P, W, C>>(&mut self, pc: T);

    fn has(&self, point: &Point3D<P>) -> bool;

    fn batch(&mut self, f: fn(&mut Voxel<C, W>));
}


pub struct PointCloud<P: Number, W: UInt, C: UInt> {
    _phantom: PhantomData<W>,
    pub field: Vec<(Point3D<P>, Voxel<C, W>)>,
    pub bounds: Option<(Point3D<P>, Point3D<P>)>,
    pub offset: Point3D<P>,
}

impl<P: Number, W: UInt, C: UInt> Default for PointCloud<P, W, C> {
    fn default() -> Self {
        PointCloud {
            _phantom: PhantomData,
            field: Vec::default(),
            bounds: None,
            offset: Point3D::default(),
        }
    }
}

impl<P: Number, W: UInt, C: UInt> PrivateVoxelCollectionMethod<P, W, C> for PointCloud<P, W, C> {
    fn get_inner_bounds(&self) -> Option<(Point3D<P>, Point3D<P>)> {
        self.bounds
    }

    fn set_inner_bounds(&mut self, bounds: (Point3D<P>, Point3D<P>)) {
        self.bounds = Some(bounds)
    }
}

impl<P, W, C> VoxelCollection<P, W, C> for PointCloud<P, W, C>
where
    P: Number,
    C: UInt + AsPrimitive<W>,
    W: UInt + AsPrimitive<C>,
{
    fn with_bounds_and_offset(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>) -> Self {
        Self {
            _phantom: PhantomData,
            field: voxels,
            bounds,
            offset,
        }
    }

    fn has_bounds(&self) -> bool {
        self.bounds.is_some()
    }

    fn get_offset(&self) -> Point3D<P> {
        self.offset
    }

    fn set_offset(&mut self, offset: Point3D<P>) {
        self.offset = offset;
    }

    fn to_vec(&self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field.clone()
    }

    fn into_vec(self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field
    }

    // 挿入し、colorとweightを加算する
    fn insert<T: VoxelCollection<P, W, C>>(&mut self, pc: T) {
        self.field.extend(pc.into_vec());
        self.bounds = None;
    }

    fn insert_one(&mut self, point: Point3D<P>, voxel: Voxel<C, W>) {
        self.field.push((point, voxel));
        self.bounds = None;
    }

    fn merge<T: VoxelCollection<P, W, C>>(&mut self, pc: T) {
        self.insert(pc);
    }

    fn has(&self, point: &Point3D<P>) -> bool {
        self.field.iter().any(|(p, _)| p == point)
    }

    fn batch(&mut self, f: fn(&mut Voxel<C, W>)) {
        self.field.iter_mut().for_each(|(_, voxel)| {
            f(voxel);
        });
    }
}


/// 内部的に2次元の`Vec<Vec<Vec<Voxel>>>`を使用してボクセルの集合を表現している。
pub struct Vec3VoxelCollection<P, W, C>
where
    P: Int,
    W: UInt,
    C: UInt,
{
    pub field: Vec<Vec<Vec<Voxel<C, W>>>>,
    bounds: (Point3D<P>, Point3D<P>),
    offset: Point3D<P>,
}


impl<P, W, C> Default for Vec3VoxelCollection<P, W, C>
where
    P: Int,
    W: UInt,
    C: UInt,
{
    fn default() -> Self {
        Vec3VoxelCollection {
            field: Vec::default(),
            bounds: (Point3D::default(), Point3D::default()),
            offset: Point3D::<P>::default(),
        }
    }
}

impl<P: Int, W: UInt, C: UInt> PrivateVoxelCollectionMethod<P, W, C> for Vec3VoxelCollection<P, W, C> {
    fn get_inner_bounds(&self) -> Option<(Point3D<P>, Point3D<P>)> {
        Some(self.bounds)
    }

    fn set_inner_bounds(&mut self, _bounds: (Point3D<P>, Point3D<P>)) {
        panic!("Vec3VoxelCollection can't set bounds manually")
    }
}

impl<P, W, C> VoxelCollection<P, W, C> for Vec3VoxelCollection<P, W, C>
where
    P: Int + AsPrimitive<usize>,
    C: UInt + AsPrimitive<W>,
    W: UInt + AsPrimitive<C>,
    usize: AsPrimitive<P>,
{
    fn with_bounds_and_offset(points: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>) -> Self {
        let (min, max) = bounds.unwrap_or_else(|| {
            Self::calc_bounds(&points)
        });


        let field_size = max - min + P::one();

        let mut field = vec![vec![vec![Voxel::<C, W>::default(); field_size[2].as_()]; field_size[1].as_()]; field_size[0].as_()];


        points.into_iter().for_each(|(point, voxel)| {
            let point = point - min;

            let x: usize = point[0].as_();
            let y: usize = point[1].as_();
            let z: usize = point[2].as_();

            field[x][y][z] = voxel;
        });

        Self {
            field,
            bounds: (min, max),
            offset,
        }
    }
    fn has_bounds(&self) -> bool {
        true
    }

    fn get_offset(&self) -> Point3D<P> {
        self.offset
    }

    fn set_offset(&mut self, offset: Point3D<P>) {
        self.offset = offset;
    }

    fn to_vec(&self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field.iter().enumerate().flat_map(|(x, y_vec)| {
            y_vec.iter().enumerate().flat_map(move |(y, z_vec)| {
                z_vec.iter().enumerate().map(move |(z, voxel)| {
                    let point = Point3D::new([x, y, z]).as_() + self.bounds.0;
                    (point, *voxel)
                })
            })
        }).collect()
    }

    fn into_vec(self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field.into_iter().enumerate().flat_map(|(x, y_vec)| {
            y_vec.into_iter().enumerate().flat_map(move |(y, z_vec)| {
                z_vec.into_iter().enumerate().map(move |(z, voxel)| {
                    let point = Point3D::new([x, y, z]).as_() + self.bounds.0;
                    (point, voxel)
                })
            })
        }).collect()
    }


    fn insert_one(&mut self, point: Point3D<P>, voxel: Voxel<C, W>) {
        let point = point - self.bounds.0;
        let x: usize = point[0].as_();
        let y: usize = point[1].as_();
        let z: usize = point[2].as_();

        if let Some(y_vec) = self.field.get_mut(x) {
            if let Some(z_vec) = y_vec.get_mut(y) {
                if let Some(current_voxel) = z_vec.get_mut(z) {
                    Self::add_color_with_weight_check(current_voxel, voxel);
                }
            }
        }
    }

    fn merge<T: VoxelCollection<P, W, C>>(&mut self, mut pc: T) {
        let new_bounds = Self::calc_bounds_from_2(self.get_bounds(), pc.get_bounds());

        if self.get_bounds() == new_bounds {
            self.insert(pc);
        } else {
            *self = Self::with_bounds(self.to_vec(),new_bounds);
            self.insert(pc);
        }
    }

    fn has(&self, point: &Point3D<P>) -> bool {
        let point = *point - self.bounds.0;
        let x: usize = point[0].as_();
        let y: usize = point[1].as_();
        let z: usize = point[2].as_();

        if let Some(y_vec) = self.field.get(x) {
            if let Some(z_vec) = y_vec.get(y) {
                if let Some(voxel) = z_vec.get(z) {
                    return voxel.weight.ne(&W::zero());
                }
            }
        }
        false
    }

    fn batch(&mut self, f: fn(&mut Voxel<C, W>)) {
        self.field.iter_mut().for_each(|y_vec| {
            y_vec.iter_mut().for_each(|z_vec| {
                z_vec.iter_mut().for_each(|voxel| {
                    f(voxel);
                });
            });
        });
    }
}


/// 内部的に2次元の`Vec<Vec<(Height, Voxel)>>`を使用してボクセルの集合を表現している。
/// ある平面座標において、異なる高さのボクセルは格納できない。
/// 地表面など、ある平面座標において高さが一意に定まる場合に使用する。
pub struct Vec2VoxelCollection<P, W, C>
where
    P: Int,
    W: UInt,
    C: UInt,
{
    pub field: Vec<Vec<(P, Voxel<C, W>)>>,
    bounds_xy: (Point2D<P>, Point2D<P>),
    bounds_z: Option<(P, P)>,
    offset: Point3D<P>,
}

impl<P, W, C> Vec2VoxelCollection<P, W, C>
where
    P: Int,
    W: UInt,
    C: UInt,
{
    pub fn get_bounds_xy(&self) -> (Point2D<P>, Point2D<P>) {
        self.bounds_xy
    }
}

impl<P, W, C> Default for Vec2VoxelCollection<P, W, C>
where
    P: Int,
    W: UInt,
    C: UInt,
{
    fn default() -> Self {
        Vec2VoxelCollection {
            field: Vec::default(),
            bounds_xy: (Point2D::default(), Point2D::default()),
            bounds_z: Some((P::default(), P::default())),
            offset: Point3D::<P>::default(),
        }
    }
}

impl<P, W, C> PrivateVoxelCollectionMethod<P, W, C> for Vec2VoxelCollection<P, W, C>
where
    P: Int,
    W: UInt,
    C: UInt,
{
    fn get_inner_bounds(&self) -> Option<(Point3D<P>, Point3D<P>)> {
        if let Some((min_z, max_z)) = self.bounds_z {
            let (min_xy, max_xy) = self.bounds_xy;

            Some((Point3D::new([min_xy[0], min_xy[1], min_z]), Point3D::new([max_xy[0], max_xy[1], max_z])))
        }else {
            None
        }
    }
    fn set_inner_bounds(&mut self, _bounds: (Point3D<P>, Point3D<P>)) {
        panic!("Vec2VoxelCollection can't set bounds manually")
    }
}

impl<P, W, C> VoxelCollection<P, W, C> for Vec2VoxelCollection<P, W, C>
where
    P: Int + AsPrimitive<usize>,
    C: UInt + AsPrimitive<W>,
    W: UInt + AsPrimitive<C>,
    usize: AsPrimitive<P>,
{
    fn with_bounds_and_offset(points: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>) -> Self {
        let (min, max) = bounds.unwrap_or_else(|| {
            Self::calc_bounds(&points)
        });

        let field_size = max - min + P::one();

        let mut field = vec![vec![(P::default(), Voxel::<C, W>::default()); field_size[1].as_()]; field_size[0].as_()];

        points.into_iter().for_each(|(point, voxel)| {
            let x: usize = (point[0] - min[0]).as_();
            let y: usize = (point[1] - min[1]).as_();
            let z = point[2];

            field[x][y] = (z, voxel);
        });

        Self {
            field,
            bounds_xy:(min.fit(),max.fit()),
            bounds_z: Some((min[2], max[2])),
            offset,
        }
    }

    fn has_bounds(&self) -> bool {
        true
    }

    fn get_bounds(&mut self) -> (Point3D<P>, Point3D<P>) {
        let bounds_z = self.bounds_z.unwrap_or_else(||{
            let z = self.field.iter().flat_map(|y_vec| {
                y_vec.iter().filter_map(|(z, voxel)| {
                    if voxel.weight.ne(&W::zero()) {
                        Some(*z)
                    } else {
                        None
                    }
                })
            }).collect::<Vec<_>>();
            
            let min = z.iter().min().unwrap();
            let max = z.iter().max().unwrap();

            (*min, *max)
        });
        
        let min_xy = self.bounds_xy.0;
        let max_xy = self.bounds_xy.1;
        
        let min = Point3D::new([min_xy[0], min_xy[1], bounds_z.0]);
        let max = Point3D::new([max_xy[0], max_xy[1], bounds_z.1]);
        
        (min, max)
    }

    fn get_offset(&self) -> Point3D<P> {
        self.offset
    }

    fn set_offset(&mut self, offset: Point3D<P>) {
        self.offset = offset;
    }

    fn to_vec(&self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field.iter().enumerate().flat_map(|(x, y_vec)| {
            y_vec.iter().enumerate().map(move |(y, (z, voxel))| {
                let min_xy = self.bounds_xy.0;

                let x = x.as_() + min_xy[0];
                let y = y.as_() + min_xy[1];

                let point = Point3D::new([x, y, *z]);
                (point, *voxel)
            })
        }).collect()
    }

    fn into_vec(self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field.into_iter().enumerate().flat_map(|(x, y_vec)| {
            y_vec.into_iter().enumerate().map(move |(y, (z, voxel))| {
                let min_xy = self.bounds_xy.0;

                let x = x.as_() + min_xy[0];
                let y = y.as_() + min_xy[1];

                let point = Point3D::new([x, y, z]);
                (point, voxel)
            })
        }).collect()
    }

    fn insert_one(&mut self, point: Point3D<P>, voxel: Voxel<C, W>) {
        let x: usize = (point[0] - self.bounds_xy.0[0]).as_();
        let y: usize = (point[1] - self.bounds_xy.0[1]).as_();
        let z = point[2];
        if let Some(y_vec) = self.field.get_mut(x) {
            if let Some((height, current_voxel)) = y_vec.get_mut(y) {
                if *height != z {
                    *height = z;
                    *current_voxel = voxel;

                    return;
                }

                Self::add_color_with_weight_check(current_voxel, voxel);
            }
        }

        self.bounds_z = None;
    }

    fn merge<T: VoxelCollection<P, W, C>>(&mut self, mut pc: T) {
        let new_bounds = Self::calc_bounds_from_2(self.get_bounds(), pc.get_bounds());

        if self.get_bounds() == new_bounds {
            self.insert(pc);
        } else {
            *self = Self::with_bounds(self.to_vec(), new_bounds);
            self.insert(pc);
        }
    }

    fn has(&self, point: &Point3D<P>) -> bool {
        let x: usize = point[0].as_();
        let y: usize = point[1].as_();
        let z = point[2];
        if let Some(y_vec) = self.field.get(x) {
            if let Some((height, voxel)) = y_vec.get(y) {
                return *height == z && voxel.weight.ne(&W::zero());
            }
        }
        false
    }

    fn batch(&mut self, f: fn(&mut Voxel<C, W>)) {
        self.field.iter_mut().for_each(|y_vec| {
            y_vec.iter_mut().for_each(|(_z, voxel)| {
                f(voxel);
            });
        });
    }
}

/// 内部的に`DashMap<Point3D, Voxel, BuildHasherDefault<FxHasher>>`を使用してボクセルの集合を表現している。
pub struct HMap3DVoxelCollection<P, W, C, BH>
where
    P: Int,
    W: UInt,
    C: UInt,
    BH: BuildHasher,
{
    pub field: DashMap<Point3D<P>, Voxel<C, W>, BH>,
    bounds: Option<(Point3D<P>, Point3D<P>)>,
    offset: Point3D<P>,
}

impl<P, W, C, BH> Default for HMap3DVoxelCollection<P, W, C, BH>
where
    P: Int,
    W: UInt,
    C: UInt,
    BH: BuildHasher + Clone + Default,
{
    fn default() -> Self {
        HMap3DVoxelCollection {
            field: DashMap::with_hasher(BH::default()),
            bounds: None,
            offset: Point3D::<P>::default(),
        }
    }
}

impl<P, W, C, BH> PrivateVoxelCollectionMethod<P, W, C> for HMap3DVoxelCollection<P, W, C, BH>
where
    P: Int,
    W: UInt,
    C: UInt,
    BH: BuildHasher + Clone + Default,
{
    fn get_inner_bounds(&self) -> Option<(Point3D<P>, Point3D<P>)> {
        self.bounds
    }

    fn set_inner_bounds(&mut self, bounds: (Point3D<P>, Point3D<P>)) {
        self.bounds = Some(bounds);
    }
}

impl<P, W, C, BH> VoxelCollection<P, W, C> for HMap3DVoxelCollection<P, W, C, BH>
where
    BH: BuildHasher + Clone + Default,
    P: Int,
    C: UInt + AsPrimitive<W>,
    W: UInt + AsPrimitive<C>,
{
    fn with_bounds_and_offset(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>) -> Self {

        let field = DashMap::<Point3D<P>, Voxel<C, W>, BH>::with_hasher(BH::default());

        let voxels = voxels.into_iter().map(|(point, voxel)| { (point, voxel) }).collect::<Vec<_>>();

        voxels.into_iter().for_each(|(point, voxel)| {
            field.entry(point).and_modify(|current_voxel| {
                Self::add_color_with_weight_check(current_voxel, voxel);
            }).or_insert(voxel);
        });


        Self {
            field,
            bounds,
            offset,
        }
    }

    fn has_bounds(&self) -> bool {
        self.bounds.is_some()
    }
    fn get_offset(&self) -> Point3D<P> {
        self.offset
    }

    fn set_offset(&mut self, offset: Point3D<P>) {
        self.offset = offset;
    }

    fn to_vec(&self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field.clone().into_iter().map(|entry| {
            let (point, voxel) = entry;
            (point, voxel)
        }).collect()
    }

    fn into_vec(self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field.into_iter().map(|entry| {
            let (point, voxel) = entry;
            (point, voxel)
        }).collect()
    }

    fn insert_one(&mut self, point: Point3D<P>, voxel: Voxel<C, W>) {
        self.field.entry(point).and_modify(|current_voxel| {
            Self::add_color_with_weight_check(current_voxel, voxel);
        }).or_insert(voxel);

        self.bounds = None;
    }

    fn merge<T: VoxelCollection<P, W, C>>(&mut self, pc: T) {
        self.insert(pc)
    }

    fn has(&self, point: &Point3D<P>) -> bool {
        self.field.contains_key(point)
    }

    fn batch(&mut self, f: fn(&mut Voxel<C, W>)) {
        self.field.iter_mut().for_each(|mut entry| {
            let (_point, voxel) = entry.pair_mut();
            f(voxel);
        });
    }
}

/// 内部的に`DashMap<Point2D, (Height, Voxel), BuildHasherDefault<FxHasher>>`を使用してボクセルの集合を表現している。
/// ある平面座標において、異なる高さのボクセルは格納できない。
/// 地表面など、ある平面座標において高さが一意に定まる場合に使用する。
pub struct HMap2DVoxelCollection<P, W, C, BH>
where
    P: UInt,
    W: UInt,
    C: UInt,
    BH: BuildHasher,
{
    pub field: DashMap<Point2D<P>, (P, Voxel<C, W>), BH>,
    bounds: Option<(Point3D<P>, Point3D<P>)>,
    offset: Point3D<P>,
}

impl<P, W, C, BH> Default for HMap2DVoxelCollection<P, W, C, BH>
where
    P: UInt,
    W: UInt,
    C: UInt,
    BH: BuildHasher + Clone + Default,
{
    fn default() -> Self {
        HMap2DVoxelCollection {
            field: DashMap::with_hasher(BH::default()),
            bounds: None,
            offset: Point3D::<P>::default(),
        }
    }
}

impl<P, W, C, BH> PrivateVoxelCollectionMethod<P, W, C> for HMap2DVoxelCollection<P, W, C, BH>
where
    P: UInt,
    W: UInt,
    C: UInt,
    BH: BuildHasher + Clone + Default,
{
    fn get_inner_bounds(&self) -> Option<(Point3D<P>, Point3D<P>)> {
        self.bounds
    }

    fn set_inner_bounds(&mut self, bounds: (Point3D<P>, Point3D<P>)) {
        self.bounds = Some(bounds);
    }
}

impl<P, W, C, BH> VoxelCollection<P, W, C> for HMap2DVoxelCollection<P, W, C, BH>
where
    P: UInt,
    W: UInt + AsPrimitive<C>,
    C: UInt + AsPrimitive<W>,
    BH: BuildHasher + Clone + Default,
{
    fn with_bounds_and_offset(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>) -> Self {

        let field = DashMap::<Point2D<P>, (P, Voxel<C, W>), BH>::with_hasher(BH::default());

        voxels.into_iter().for_each(|(point, voxel)| {

            field.entry(point.fit()).and_modify(|(h, current_voxel)| {
                if *h == point[2] {
                    Self::add_color_with_weight_check(current_voxel, voxel);
                }
            }).or_insert((point[2], voxel));
        });


        Self {
            field,
            bounds,
            offset,
        }
    }
    fn has_bounds(&self) -> bool {
        self.bounds.is_some()
    }

    fn get_offset(&self) -> Point3D<P> {
        self.offset
    }

    fn set_offset(&mut self, offset: Point3D<P>) {
        self.offset = offset;
    }

    fn to_vec(&self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field.clone().into_iter().map(|entry| {
            let (point, voxel) = entry;
            (Point3D::new([point[0], point[1], voxel.0]), voxel.1)
        }).collect()
    }

    fn into_vec(self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        self.field.into_iter().map(|entry| {
            let (point, (z, voxel)) = entry;
            let x = point[0];
            let y = point[1];
            (Point3D::new([x, y, z]), voxel)
        }).collect()
    }

    fn insert_one(&mut self, point: Point3D<P>, voxel: Voxel<C, W>) {
        self.field.entry(point.fit()).and_modify(|(h, current_voxel)| {
            if *h == point[2] {
                Self::add_color_with_weight_check(current_voxel, voxel);
            }
        }).or_insert((point[2], voxel));

        self.bounds = None;
    }

    fn merge<T: VoxelCollection<P, W, C>>(&mut self, pc: T) {
        self.insert(pc)
    }

    fn has(&self, point: &Point3D<P>) -> bool {
        let x = point[0];
        let y = point[1];
        let z = point[2];
        let point2d = Point2D::new([x, y]);
        if let Some(r) = self.field.get(&point2d) {
            let (_key, (height, _voxel)) = r.pair();
            return *height == z;
        }
        false
    }

    fn batch(&mut self, f: fn(&mut Voxel<C, W>)) {
        self.field.iter_mut().for_each(|mut entry| {
            let (_point, (_height, voxel)) = entry.pair_mut();
            f(voxel);
        });
    }
}
