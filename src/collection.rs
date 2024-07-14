use std::hash::BuildHasher;
use std::marker::PhantomData;
use std::vec;

use anyhow::anyhow;
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

pub struct BuildVoxelCollection<P, W, C, VC>
where
    P: Number,
    W: UInt,
    C: UInt,
    VC: VoxelCollection<P, W, C>,
{
    _phantom: PhantomData<VC>,
    voxels: Vec<(Point3D<P>, Voxel<C, W>)>,
    bounds: Option<(Point3D<P>, Point3D<P>)>,
    offset: Point3D<P>,
    resolution: f64,
}

/// ボクセルの集合を構築するためのビルダーです。
impl<P, W, C, VC> BuildVoxelCollection<P, W, C, VC>
where
    P: Number,
    W: UInt,
    C: UInt,
    VC: VoxelCollection<P, W, C>,
{
    /// <必須1>
    /// 座標とボクセルのペアのリストを指定します。
    /// このメソッドの代わりに`points`メソッドを使用することもできます。
    /// 二度目以降の呼び出しでは、前回の呼び出しで指定された値は破棄されます。
    pub fn voxels(mut self, voxels: Vec<(Point3D<P>, Voxel<C, W>)>) -> Self {
        self.voxels = voxels;
        self
    }

    /// <必須2>
    /// 座標と色のペアのリストを指定します。
    /// このメソッドの代わりに`voxels`メソッドを使用することもできます。
    /// 二度目以降の呼び出しでは、前回の呼び出しで指定された値は破棄されます。
    pub fn points(mut self, points: Vec<(Point3D<P>, Color<C>)>) -> Self {
        self.voxels = points.into_iter().map(|(point, color)| {
            (point, Voxel::new(color))
        }).collect();
        self
    }

    /// <任意>
    /// 登録されているすべての座標値から、最小値と最大値を計算して設定します。
    /// このメソッドを使用しない場合、最小値と最大値が必要になった段階で自動的に計算されます。
    /// すでに分かっている場合を除いて、わざわざ計算する必要はありません。
    pub fn bounds(mut self, bounds: (Point3D<P>, Point3D<P>)) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// <任意>
    /// 座標値のオフセットを指定します。
    pub fn offset(mut self, offset: Point3D<P>) -> Self {
        self.offset = offset;
        self
    }

    /// <任意>
    /// ボクセルの分解能を指定します。
    /// このメソッドを使用しない場合、デフォルト値は1です。
    pub fn resolution(mut self, resolution: f64) -> Self {
        self.resolution = resolution;
        self
    }

    /// 登録した内容を指定して、`VoxelCollection`を構築します。
    pub fn build(self) -> VC {
        VC::new(self.voxels, self.bounds, self.offset, self.resolution)
    }
}

///　ボクセルの集合や点群を操作するためのトレイトです。
pub trait VoxelCollection<P, W, C>: PrivateVoxelCollectionMethod<P, W, C> + Default + Clone
where
    P: Number,
    W: UInt,
    C: UInt,
    Self: Sized,
{
    /// インスタンスを生成するためのビルダーを返します。
    fn builder() -> BuildVoxelCollection<P, W, C, Self> {
        BuildVoxelCollection {
            _phantom: PhantomData,
            voxels: Vec::default(),
            bounds: None,
            offset: Point3D::default(),
            resolution: 1.,
        }
    }

    /// すべての値を明示的に指定してインスタンスを生成します。
    fn new(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>, resolution: f64) -> Self;

    /// 現時点で境界が計算されてているかどうかを返します。
    fn has_bounds(&self) -> bool;

    /// 境界がすでに計算されている場合は、その値を返します。
    /// まだ計算されていない場合は、計算し、結果を保持します。
    fn get_bounds(&mut self) -> (Point3D<P>, Point3D<P>) {
        self.get_inner_bounds().unwrap_or_else(|| {
            let (min, max) = Self::calc_bounds(&self.to_vec());
            let bounds = (min + self.get_offset(), max + self.get_offset());

            self.set_inner_bounds(bounds);

            bounds
        })
    }

    /// 分解能を返します。
    fn get_resolution(&self) -> f64;

    /// オフセットを返します。
    fn get_offset(&self) -> Point3D<P>;

    /// オフセットを設定します。
    /// 現在の値に加算されるのではなく、新しい値に置き換えられます。
    fn set_offset(&mut self, offset: Point3D<P>);

    /// 登録されているすべての座標値のうち、各軸の最小値に合わせてオフセットを調整します。
    /// すでに境界値が計算されている場合はその値を利用し、計算されていない場合は新たに計算して結果を保持します。
    fn offset_to_min(&mut self) {
        let current_offset = self.get_offset();

        let (min, _max) = self.get_bounds();

        if current_offset != min {
            let offset = min - current_offset;
            self.set_offset(offset);
        }
    }

    /// 登録されているすべての座標とボクセルのタプルを返します。
    /// オフセットを適用した結果を得たい場合、`to_vec_with_offset`メソッドを使用してください。
    /// `Voxel`の場合、`Voxel.color / Voxel.weight`で平均色を計算できます。
    /// すでに平均が計算された色を取得したい場合は、`to_points`メソッドを使用してください。
    fn to_vec(&self) -> Vec<(Point3D<P>, Voxel<C, W>)>;

    /// 登録されているすべての座標とボクセルのタプルを返します。
    /// オフセットを適用した結果を得たい場合、`into_vec_with_offset`メソッドを使用してください。
    /// `Voxel`の場合、`Voxel.color / Voxel.weight`で平均色を計算できます。
    /// すでに平均が計算された色を取得したい場合は、`into_points`メソッドを使用してください。
    fn into_vec(self) -> Vec<(Point3D<P>, Voxel<C, W>)>;

    /// 登録されているすべての座標とボクセルのタプルを返します。
    /// 座標値には設定されているオフセットが加算されます。
    /// すでに平均が計算された色を取得したい場合は、`to_points_with_offset`メソッドを使用してください。
    fn to_vec_with_offset(&self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        let offset = self.get_offset();
        self.to_vec().into_iter().map(|(point, voxel)| {
            (point + offset, voxel)
        }).collect()
    }

    /// 登録されているすべての座標とボクセルのタプルを返します。
    /// 座標値には設定されているオフセットが加算されます。
    /// すでに平均が計算された色を取得したい場合は、`into_points_with_offset`メソッドを使用してください。
    fn into_vec_with_offset(self) -> Vec<(Point3D<P>, Voxel<C, W>)> {
        let offset = self.get_offset();
        self.into_vec().into_iter().map(|(point, voxel)| {
            (point + offset, voxel)
        }).collect()
    }

    /// 登録されているすべてのボクセルの座標と、その色を返します。
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

    /// 登録されているすべてのボクセルの座標と、その色を返します。
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

    /// 登録されているすべてのボクセルの座標にオフセットを加算したものと、その色のタプルを返します。
    fn to_points_with_offset(&self) -> Vec<(Point3D<P>, Color<C>)>
    where
        C: AsPrimitive<W>,
        W: AsPrimitive<C>,
    {
        self.to_vec_with_offset().into_iter().map(|(point, voxel)| {
            let average_color = voxel.color / Color::from(voxel.weight).as_::<C>();
            (point, average_color)
        }).collect()
    }

    /// 登録されているすべてのボクセルの座標にオフセットを加算したものと、その色のタプルを返します。
    fn into_points_with_offset(self) -> Vec<(Point3D<P>, Color<C>)>
    where
        C: AsPrimitive<W>,
        W: AsPrimitive<C>,
    {
        self.into_vec_with_offset().into_iter().map(|(point, voxel)| {
            let average_color = voxel.color / Color::from(voxel.weight).as_::<C>();
            (point, average_color)
        }).collect()
    }

    /// 現在のインスタンスに別の`VoxelCollection`を追加します。
    /// メソッドの使用者側は2つの`VoxelCollection`の分解能が同じであることを保証する必要があります。
    ///
    /// `VoxelCollection`の種類によって、現在の境界外の座標値は無視されることがあります。
    /// すべての値を追加したい場合、`merge`メソッドを使用してください。
    ///
    /// 以下の構造体は`insert`と`merge`の結果が同じであるため、より高速な`insert`メソッドを使用することを推奨します。
    ///
    /// + `PointCloud`
    /// + `HMap3DVoxelCollection`
    /// + `HMap2DVoxelCollection`
    fn insert<T: VoxelCollection<P, W, C>>(&mut self, pc: T) {
        pc.into_vec_with_offset().into_iter().for_each(|(point, voxel)| {
            self.insert_one(point, voxel);
        });
    }

    /// 現在のインスタンスに1点を追加します。
    /// メソッドの使用者は挿入する点の分解能と`VoxelCollection`の分解能が同じであることを保証する必要があります。
    fn insert_one(&mut self, point: Point3D<P>, voxel: Voxel<C, W>);

    /// 現在のインスタンスに別の`VoxelCollection`をマージします。
    /// メソッドの使用者側は2つの`VoxelCollection`の分解能が同じであることを保証する必要があります。
    /// `insert`と比較して、すべての値を確実に追加しますが、メモリの再割り当てが必要になるため低速であることが予想されます。
    ///
    /// 以下の構造体において、境界外の値を挿入する場合はこのメソッドを使用する必要があります。
    ///
    /// + `Vec3DVoxelCollection`
    /// + `Vec2DVoxelCollection`
    ///
    /// # Errors
    ///
    /// + 2つの`VoxelCollection`の分解能が異なる場合、エラーを返します。
    fn merge<T: VoxelCollection<P, W, C>>(mut self, mut pc: T) -> Result<Self, anyhow::Error> {
        if self.get_resolution() != pc.get_resolution() {
            return Err(anyhow!("Resolution is different"));
        }

        let resolution = self.get_resolution();

        let new_bounds = if self.has_bounds() && pc.has_bounds() {
            Some(Self::calc_bounds_from_2(self.get_bounds(), pc.get_bounds()))
        } else { None };

        let voxels = [
            self.into_vec_with_offset(),
            pc.into_vec_with_offset()
        ].concat();


        Ok(Self::new(voxels, new_bounds, Point3D::default(), resolution))
    }

    /// 指定された座標値が登録されているかどうかを返します。
    fn has(&self, point: &Point3D<P>) -> bool;

    /// 登録されているすべてのボクセルに対して、指定された関数を適用します。
    fn batch(&mut self, f: fn(&mut Voxel<C, W>));
}

/// ボクセルや点群の集合を表現するための構造体です。
/// 内部的に配列を使用しているため、インスタンスの生成を高速で行える一方で、隣接する座標値の検索にはすべての要素を走査する必要があります。
/// ボクセル化されていない点群を扱う場合などに使用することを推奨します。
#[derive(Clone)]
pub struct PointCloud<P: Number, W: UInt, C: UInt> {
    _phantom: PhantomData<W>,
    pub field: Vec<(Point3D<P>, Voxel<C, W>)>,
    pub bounds: Option<(Point3D<P>, Point3D<P>)>,
    pub offset: Point3D<P>,
    pub resolution: f64,
}

impl<P: Number, W: UInt, C: UInt> Default for PointCloud<P, W, C> {
    /// 分解能を`1.`として、空のインスタンスを生成します。
    fn default() -> Self {
        PointCloud {
            _phantom: PhantomData,
            field: Vec::default(),
            bounds: None,
            offset: Point3D::default(),
            resolution: 1.,
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
    fn new(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>, resolution: f64) -> Self {
        Self {
            _phantom: PhantomData,
            field: voxels,
            bounds,
            offset,
            resolution,
        }
    }

    fn has_bounds(&self) -> bool {
        self.bounds.is_some()
    }

    fn get_resolution(&self) -> f64 {
        self.resolution
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
        self.field.extend(pc.into_vec_with_offset());
        self.bounds = None;
    }

    fn insert_one(&mut self, point: Point3D<P>, voxel: Voxel<C, W>) {
        self.field.push((point, voxel));
        self.bounds = None;
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


/// 内部的に3次元配列を使用してボクセルの集合を表現するための構造体です。
/// 隣接する座標値の検索が高速で行える一方で、境界に合わせて多次元配列を構築するため多くのメモリが必要になります。
/// また、境界外の値を挿入するにはメモリの再確保を行う必要があります。
#[derive(Clone)]
pub struct Vec3VoxelCollection<P, W, C>
where
    P: Int,
    W: UInt,
    C: UInt,
{
    pub field: Vec<Vec<Vec<Voxel<C, W>>>>,
    bounds: (Point3D<P>, Point3D<P>),
    offset: Point3D<P>,
    resolution: f64,
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
            resolution: 1.,
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
    fn new(points: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>, resolution: f64) -> Self {
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
            resolution,
        }
    }
    fn has_bounds(&self) -> bool {
        true
    }

    fn get_resolution(&self) -> f64 {
        self.resolution
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

/// 内部的に2次元配列を使用してボクセルの集合を表現するための構造体です。
/// 平面座標ごとに1つの高さしか持てませんが、`Vec3VoxelCollection`よりもメモリ効率が良いです。
/// 隣接する座標値の検索を高速に行えます。
/// また、境界外の値を挿入するにはメモリの再確保を行う必要があります。
#[derive(Clone)]
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
    resolution: f64,
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
            resolution: 1.,
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
        } else {
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
    fn new(points: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>, resolution: f64) -> Self {
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
            bounds_xy: (min.fit(), max.fit()),
            bounds_z: Some((min[2], max[2])),
            offset,
            resolution,
        }
    }

    fn has_bounds(&self) -> bool {
        true
    }

    fn get_bounds(&mut self) -> (Point3D<P>, Point3D<P>) {
        let bounds_z = self.bounds_z.unwrap_or_else(|| {
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

    fn get_resolution(&self) -> f64 {
        self.resolution
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

/// 内部的にハッシュマップを使用して座標を管理しています。
/// 1点挿入するごとにハッシュ値を計算するため、`Vec3VoxelCollection`よりも低速であることが予想されますが、境界外の値を挿入する際にメモリの再確保が不要です。
/// 隣接点の検索は1回のハッシュ値計算で行えるため、`PointCloud`よりも高速です。
#[derive(Clone)]
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
    resolution: f64,
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
            resolution: 1.,
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
    fn new(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>, resolution: f64) -> Self {
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
            resolution,
        }
    }

    fn has_bounds(&self) -> bool {
        self.bounds.is_some()
    }

    fn get_resolution(&self) -> f64 {
        self.resolution
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

/// 内部的にハッシュマップを使用して平面座標と高さを管理しています。
/// 1点の平面座標に対して1つの高さしか持てないという制約があります。
/// 1点挿入するごとにハッシュ値を計算するため、`Vec2VoxelCollection`よりも低速であることが予想されますが、境界外の値を挿入する際にメモリの再確保が不要です。
/// 隣接点の検索は1回のハッシュ値計算で行えるため、`PointCloud`よりも高速です。
#[derive(Clone)]
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
    resolution: f64,
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
            resolution: 1.,
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
    fn new(voxels: Vec<(Point3D<P>, Voxel<C, W>)>, bounds: Option<(Point3D<P>, Point3D<P>)>, offset: Point3D<P>, resolution: f64) -> Self {
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
            resolution,
        }
    }
    fn has_bounds(&self) -> bool {
        self.bounds.is_some()
    }

    fn get_resolution(&self) -> f64 {
        self.resolution
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
