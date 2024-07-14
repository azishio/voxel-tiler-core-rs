use std::fmt::Debug;
use std::hash::Hash;

use coordinate_transformer::ZoomLv;
use nohash::IsEnabled;
use num::{Bounded, CheckedAdd, CheckedSub, Num};
use num::traits::NumAssignOps;
use ordered_float::OrderedFloat;
use vec_x::VecX;

/// 扱うことのできる数値型を表します。
/// Rustの浮動小数点数は`Eq`や`Hash`を実装できないため、小数表現には[`OrderedFloat`]を使用します。
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

/// 扱うことのできる整数型を表します。
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

/// 扱うことのできる符号なし整数型を表します。
pub trait UInt: Int + CheckedAdd + CheckedSub
{}

impl UInt for u8 {}

impl UInt for u16 {}

impl UInt for u32 {}

impl UInt for u64 {}

impl UInt for usize {}

/// RGB色を表します。
pub type Color<P> = VecX<P, 3>;

/// 単一のボクセルを表現します。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Voxel<C, W>
where
    C: UInt,
    W: UInt,
{
    /// 色情報です。
    /// `weight`が1より大きい場合、このボクセルの色は`color / weight`で決定されます。
    pub color: Color<C>,
    /// このボクセルが専有する空間に存在した頂点の数です。
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


/// 現在の座標値と隣接した座標値を計算します。
/// 隣接する座標値が使用している数値型の境界外の場合、`None`を返します。
pub trait Point: Clone + Copy {
    /// 1次元目の値を1増加させた座標を返します。
    fn right(&self) -> Option<Self>;

    /// 1次元目の値を1減少させた座標を返します。
    fn left(&self) -> Option<Self>;

    /// 2次元目の値を1増加させた座標を返します。
    fn front(&self) -> Option<Self>;

    /// 2次元目の値を1減少させた座標を返します。
    fn back(&self) -> Option<Self>;

    /// 3次元目の値を1増加させた座標を返します。
    /// 2次元座標の場合、上下の座標は存在しないため、常に`None`を返します。
    fn top(&self) -> Option<Self>;

    /// 3次元目の値を1減少させた座標を返します。
    /// 2次元座標の場合、上下の座標は存在しないため、常に`None`を返します。
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

/// ボクセライザーの分解能を表します。
pub enum Resolution {
    /// メートル単位の分解能です。
    Mater(f64),

    /// 平面直角座標系の点群をWebメルカトル図法で投影された地球におけるタイル座標系を使用してボクセル化する際のオプションです。
    /// 分解能は指定されたズームレベルにおけるピクセルの分解能です。
    /// 例えば、ズームレベルが`ZoomLv::Lv10`の場合、赤道上での1ピクセルの分解能は`[赤道長さ] / 2^10 / 256`です。
    /// タイル座標に関する詳細は[こちら](https://developers.google.com/maps/documentation/javascript/coordinates)を参照してください。
    Tile {
        zoom_lv: ZoomLv,
    },
}
