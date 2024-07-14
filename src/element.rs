use std::fmt::Debug;
use std::hash::Hash;

use coordinate_transformer::ZoomLv;
use nohash::IsEnabled;
use num::{Bounded, CheckedAdd, CheckedSub, Num};
use num::traits::NumAssignOps;
use ordered_float::OrderedFloat;
use vec_x::VecX;

pub trait Number: Num + Copy + Send + NumAssignOps + Default + PartialEq + Eq + PartialOrd + Ord + Hash + Bounded + Debug {}

impl Number for u8 {}

impl Number for u16 {}

impl Number for u32 {}

impl Number for u64 {}

impl Number for usize {}

impl Number for i8 {}

impl Number for i16 {}

impl Number for i32 {}

impl Number for i64 {}

impl Number for isize {}

impl Number for OrderedFloat<f32> {}

impl Number for OrderedFloat<f64> {}

pub trait Int: Number + CheckedAdd + CheckedSub + IsEnabled
{}

impl Int for u8 {}

impl Int for u16 {}

impl Int for u32 {}

impl Int for u64 {}

impl Int for usize {}

impl Int for i8 {}

impl Int for i16 {}

impl Int for i32 {}

impl Int for i64 {}

impl Int for isize {}

pub trait UInt: Int + CheckedAdd + CheckedSub
{}

impl UInt for u8 {}

impl UInt for u16 {}

impl UInt for u32 {}

impl UInt for u64 {}

impl UInt for usize {}

pub type Color<P> = VecX<P, 3>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Voxel<C, W>
where
    C: UInt,
    W: UInt,
{
    pub color: Color<C>,
    pub weight: W,
}

impl<C, W> Voxel<C, W>
where
    C: UInt,
    W: UInt,

{
    pub fn new(color: Color<C>) -> Self {
        Self { color, weight: W::one() }
    }
}


pub trait Point: Clone + Copy {
    fn right(&self) -> Option<Self>;

    fn left(&self) -> Option<Self>;

    fn front(&self) -> Option<Self>;

    fn back(&self) -> Option<Self>;

    fn top(&self) -> Option<Self>;

    fn bottom(&self) -> Option<Self>;

    /// 与えられた座標値のリストから次元ごとの`(最小値, 最大値)`を計算します。
    fn calc_bounds(list: &Vec<Self>) -> (Self, Self);
}

pub type Point2D<P> = VecX<P, 2>;

impl<P: Int> Point for Point2D<P> {
    fn right(&self) -> Option<Self> {
        let result = self[0].checked_add(&P::one())?;
        Some(Self::new([result, self[1]]))
    }

    fn left(&self) -> Option<Self> {
        let result = self[0].checked_sub(&P::one())?;
        Some(Self::new([result, self[1]]))
    }

    fn front(&self) -> Option<Self> {
        let result = self[1].checked_add(&P::one())?;
        Some(Self::new([self[0], result]))
    }

    fn back(&self) -> Option<Self> {
        let result = self[1].checked_sub(&P::one())?;
        Some(Self::new([self[0], result]))
    }

    fn top(&self) -> Option<Self> {
        None
    }

    fn bottom(&self) -> Option<Self> {
        None
    }

    fn calc_bounds(list: &Vec<Self>) -> (Self, Self) {
        let mut min = list[0];
        let mut max = list[0];

        list.into_iter().for_each(|&point| {
            min = min.batch_with(point, |a, b| a.min(b));
            max = max.batch_with(point, |a, b| a.max(b));
        });

        (min, max)
    }
}

pub type Point3D<P> = VecX<P, 3>;

impl<P: Int> Point for Point3D<P> {
    fn right(&self) -> Option<Self> {
        let result = self[0].checked_add(&P::one())?;
        Some(Self::new([result, self[1], self[2]]))
    }

    fn left(&self) -> Option<Self> {
        let result = self[0].checked_sub(&P::one())?;
        Some(Self::new([result, self[1], self[2]]))
    }

    fn front(&self) -> Option<Self> {
        let result = self[1].checked_add(&P::one())?;
        Some(Self::new([self[0], result, self[2]]))
    }

    fn back(&self) -> Option<Self> {
        let result = self[1].checked_sub(&P::one())?;
        Some(Self::new([self[0], result, self[2]]))
    }

    fn top(&self) -> Option<Self> {
        let result = self[2].checked_add(&P::one())?;
        Some(Self::new([self[0], self[1], result]))
    }

    fn bottom(&self) -> Option<Self> {
        let result = self[2].checked_sub(&P::one())?;
        Some(Self::new([self[0], self[1], result]))
    }

    fn calc_bounds(list: &Vec<Self>) -> (Self, Self) {
        let mut min = list[0];
        let mut max = list[0];

        list.into_iter().for_each(|&point| {
            min = min.batch_with(point, |a, b| a.min(b));
            max = max.batch_with(point, |a, b| a.max(b));
        });

        (min, max)
    }
}

pub enum Resolution {
    Mater(f64),
    Tile {
        zoom_lv: ZoomLv,
    },
}

pub struct Triangle<P>
where
    P: Int,
{
    pub points: [Point3D<P>; 3],
}

impl<P> Triangle<P>
where
    P: Int,
{
    pub fn new(points: [Point3D<P>; 3]) -> Self {
        Self { points }
    }
}
